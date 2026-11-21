//! The standard security handler: passwords and permissions, and encrypting each string and
//! stream with its object's key. The PDF version picks the method: 1.3 RC4 40-bit, 1.4 and
//! 1.5 RC4 128-bit, 1.6 and 1.7 AES-128, 1.7ext3 AES-256 (revision 5).

use std::cell::Cell;

use aes::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use md5::{Digest, Md5};
use sha2::Sha256;

use super::info::PdfVersion;

/// Passwords and what a reader opening with the user password may do.
#[derive(Clone, Debug, Default)]
pub struct Encryption {
    pub user_password: Option<String>,
    pub owner_password: Option<String>,
    pub permissions: Permissions,
    /// Random bytes from the caller (the engine has no random source) for keys, salts and
    /// AES initialization vectors. Without them, they are derived from the document.
    pub seed: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Printing {
    #[default]
    None,
    LowResolution,
    HighResolution,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Permissions {
    pub printing: Printing,
    pub modifying: bool,
    pub copying: bool,
    pub annotating: bool,
    pub filling_forms: bool,
    pub content_accessibility: bool,
    pub document_assembly: bool,
}

const PAD: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

pub struct Crypt {
    /// 1, 2, 4 or 5.
    pub v: u8,
    pub r: u8,
    key: Vec<u8>,
    pub o: Vec<u8>,
    pub u: Vec<u8>,
    pub oe: Vec<u8>,
    pub ue: Vec<u8>,
    pub perms: Vec<u8>,
    pub p: i32,
    seed: [u8; 32],
    counter: Cell<u64>,
}

impl Crypt {
    pub fn new(e: &Encryption, version: PdfVersion, id: &[u8; 16]) -> Crypt {
        let (v, r, bits) = match version {
            PdfVersion::V1_3 => (1, 2, 40),
            PdfVersion::V1_4 | PdfVersion::V1_5 => (2, 3, 128),
            PdfVersion::V1_6 | PdfVersion::V1_7 => (4, 4, 128),
            PdfVersion::V1_7Ext3 => (5, 5, 256),
        };
        let p = permissions(&e.permissions, r);
        let seed = e.seed.unwrap_or_else(|| Sha256::digest(id).into());
        let user = e.user_password.clone().unwrap_or_default();
        let owner = e
            .owner_password
            .clone()
            .filter(|o| !o.is_empty())
            .unwrap_or_else(|| user.clone());
        let mut c = Crypt {
            v,
            r,
            key: Vec::new(),
            o: Vec::new(),
            u: Vec::new(),
            oe: Vec::new(),
            ue: Vec::new(),
            perms: Vec::new(),
            p,
            seed,
            counter: Cell::new(0),
        };
        if r == 5 {
            c.revision5(&user, &owner);
        } else {
            c.revision2to4(&user, &owner, bits / 8, id);
        }
        c
    }

    fn revision2to4(&mut self, user: &str, owner: &str, n: usize, id: &[u8; 16]) {
        let rounds = if self.r >= 3 { 50 } else { 0 };
        // The owner entry: the padded user password under a key from the owner password.
        let mut h = Md5::digest(pad(owner)).to_vec();
        for _ in 0..rounds {
            h = Md5::digest(&h[..n]).to_vec();
        }
        let mut o = rc4(&h[..n], &pad(user));
        if self.r >= 3 {
            for i in 1..=19u8 {
                o = rc4(&h[..n].iter().map(|b| b ^ i).collect::<Vec<_>>(), &o);
            }
        }
        // The file key.
        let mut m = Md5::new();
        m.update(pad(user));
        m.update(&o);
        m.update(self.p.to_le_bytes());
        m.update(id);
        let mut key = m.finalize().to_vec();
        for _ in 0..rounds {
            key = Md5::digest(&key[..n]).to_vec();
        }
        key.truncate(n);
        self.u = if self.r == 2 {
            rc4(&key, &PAD)
        } else {
            let mut u = rc4(&key, &Md5::digest([&PAD[..], id].concat()));
            for i in 1..=19u8 {
                u = rc4(&key.iter().map(|b| b ^ i).collect::<Vec<_>>(), &u);
            }
            u.resize(32, 0);
            u
        };
        self.o = o;
        self.key = key;
    }

    fn revision5(&mut self, user: &str, owner: &str) {
        let key = self.random(32);
        let (user, owner) = (truncate(user), truncate(owner));
        let salts = self.random(16);
        self.u = [&Sha256::digest([user, &salts[..8]].concat())[..], &salts].concat();
        self.ue = aes256_raw(&Sha256::digest([user, &salts[8..]].concat()), &key);
        let salts = self.random(16);
        self.o = [
            &Sha256::digest([owner, &salts[..8], &self.u].concat())[..],
            &salts,
        ]
        .concat();
        self.oe = aes256_raw(
            &Sha256::digest([owner, &salts[8..], &self.u].concat()),
            &key,
        );
        let mut perms = [0u8; 16];
        perms[..4].copy_from_slice(&self.p.to_le_bytes());
        perms[4..8].fill(0xFF);
        perms[8..12].copy_from_slice(b"Tadb");
        perms[12..].copy_from_slice(&self.random(4));
        self.perms = aes256_raw(&key, &perms);
        self.key = key;
    }

    /// Encrypts a string or stream of object `id`.
    pub fn seal(&self, id: i32, data: &[u8]) -> Vec<u8> {
        if self.v == 5 {
            return aes_cbc(&self.key, &self.random(16), data);
        }
        let mut m = Md5::new();
        m.update(&self.key);
        m.update(&id.to_le_bytes()[..3]);
        m.update([0, 0]);
        if self.v == 4 {
            m.update(b"sAlT");
        }
        let key = m.finalize();
        let key = &key[..(self.key.len() + 5).min(16)];
        if self.v == 4 {
            aes_cbc(key, &self.random(16), data)
        } else {
            rc4(key, data)
        }
    }

    /// Bytes no one can predict without the seed, different on every call.
    fn random(&self, n: usize) -> Vec<u8> {
        let i = self.counter.get();
        self.counter.set(i + 1);
        Sha256::digest([&self.seed[..], &i.to_le_bytes()].concat())[..n].to_vec()
    }
}

fn pad(password: &str) -> [u8; 32] {
    let mut out = PAD;
    let bytes: Vec<u8> = password
        .chars()
        .map(|c| if (c as u32) < 256 { c as u8 } else { b'?' })
        .take(32)
        .collect();
    out[..bytes.len()].copy_from_slice(&bytes);
    out[bytes.len()..].copy_from_slice(&PAD[..32 - bytes.len()]);
    out
}

/// UTF-8, at most 127 bytes.
fn truncate(password: &str) -> &[u8] {
    &password.as_bytes()[..password.len().min(127)]
}

fn permissions(p: &Permissions, r: u8) -> i32 {
    let mut bits: u32 = if r == 2 { 0xFFFF_FFC0 } else { 0xFFFF_F0C0 };
    let mut set = |on: bool, bit: u32| {
        if on {
            bits |= bit;
        }
    };
    set(p.printing != Printing::None, 1 << 2);
    set(p.modifying, 1 << 3);
    set(p.copying, 1 << 4);
    set(p.annotating, 1 << 5);
    if r >= 3 {
        set(p.filling_forms, 1 << 8);
        set(p.content_accessibility, 1 << 9);
        set(p.document_assembly, 1 << 10);
        set(p.printing == Printing::HighResolution, 1 << 11);
    }
    bits as i32
}

pub fn rc4(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut s: [u8; 256] = std::array::from_fn(|i| i as u8);
    let mut j = 0u8;
    for i in 0..256 {
        j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
        s.swap(i, j as usize);
    }
    let (mut i, mut j) = (0u8, 0u8);
    data.iter()
        .map(|b| {
            i = i.wrapping_add(1);
            j = j.wrapping_add(s[i as usize]);
            s.swap(i as usize, j as usize);
            b ^ s[s[i as usize].wrapping_add(s[j as usize]) as usize]
        })
        .collect()
}

/// AES-CBC with PKCS#7 padding, the IV written first.
fn aes_cbc(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    let n = 16 - data.len() % 16;
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat_n(n as u8, n));
    let mut out = iv.to_vec();
    let mut prev: [u8; 16] = iv.try_into().unwrap();
    for chunk in padded.chunks(16) {
        let mut block: [u8; 16] = std::array::from_fn(|k| chunk[k] ^ prev[k]);
        encrypt(key, &mut block);
        out.extend_from_slice(&block);
        prev = block;
    }
    out
}

/// AES-256 over whole blocks with a zero IV and no padding (key wrapping in revision 5).
fn aes256_raw(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut prev = [0u8; 16];
    data.chunks(16)
        .flat_map(|chunk| {
            let mut block: [u8; 16] = std::array::from_fn(|k| chunk[k] ^ prev[k]);
            encrypt(key, &mut block);
            prev = block;
            block
        })
        .collect()
}

fn encrypt(key: &[u8], block: &mut [u8; 16]) {
    let b = GenericArray::from_mut_slice(block);
    match key.len() {
        32 => aes::Aes256::new(GenericArray::from_slice(key)).encrypt_block(b),
        _ => aes::Aes128::new(GenericArray::from_slice(key)).encrypt_block(b),
    }
}
