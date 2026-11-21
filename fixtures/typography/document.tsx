// Two pages of text: a registered TTF family with three faces beside the built-in Helvetica, Times
// and Courier; size, weight, style, line height, alignment including justify, letter and word
// spacing, transforms, decorations, indent, ellipsis, sub and superscript, nested runs with links,
// hyphenation (default, soft hyphens, a custom callback), an emoji drawn from an image source, and
// right-to-left Arabic and Hebrew with bidi reordering and mirrored brackets.
import { Document, Font, Link, Page, StyleSheet, Text, View } from '@axonpdf/react';

// Paths are relative to fixtures/, the working directory while a fixture renders.
Font.register({
  family: 'Lato',
  fonts: [
    { src: 'assets/Lato-Regular.ttf' },
    { src: 'assets/Lato-Bold.ttf', fontWeight: 'bold' },
    { src: 'assets/Lato-Italic.ttf', fontStyle: 'italic' },
  ],
});
Font.register({ family: 'Noto Sans Arabic', src: 'assets/NotoSansArabic-Regular.ttf' });
Font.register({ family: 'Noto Sans Hebrew', src: 'assets/NotoSansHebrew-Regular.ttf' });
Font.registerEmojiSource({ builder: (code: string) => `assets/emoji/${code}.png` });

const TEXT =
  'The quick brown fox jumps over the lazy dog while the committee deliberates on internationalisation, ' +
  'interoperability and the extraordinarily uncharacteristic responsibilities of its representatives.';

const s = StyleSheet.create({
  page: { padding: 36, fontFamily: 'Lato', fontSize: 11 },
  label: { fontFamily: 'Helvetica', fontSize: 7, color: '#888', marginTop: 10, marginBottom: 2 },
  col: { width: 150, borderLeft: '0.5pt solid #ccc', paddingLeft: 6, marginRight: 12 },
});

function Sample({ label, style, children = TEXT }: { label: string; style: object; children?: string }) {
  return (
    <View wrap={false}>
      <Text style={s.label}>{label}</Text>
      <Text style={style}>{children}</Text>
    </View>
  );
}

export default function Typography() {
  return (
    <Document title="Typography" author="Fixture author">
      <Page size="A4" style={s.page}>
        <Text style={{ fontSize: 24, fontWeight: 'bold' }}>Typography</Text>
        <Text>
          Lato regular, <Text style={{ fontWeight: 700 }}>bold</Text>, <Text style={{ fontStyle: 'italic' }}>italic</Text>,{' '}
          <Text style={{ fontWeight: 600 }}>600 picks bold</Text>, <Text style={{ fontWeight: 'light' }}>light picks regular</Text>.{' '}
          <Text style={{ fontFamily: 'Helvetica' }}>Helvetica, </Text>
          <Text style={{ fontFamily: 'Helvetica-Bold' }}>Helvetica-Bold, </Text>
          <Text style={{ fontFamily: 'Helvetica', fontStyle: 'italic' }}>Helvetica oblique, </Text>
          <Text style={{ fontFamily: 'Times-Roman' }}>Times-Roman, </Text>
          <Text style={{ fontFamily: 'Times-Roman', fontWeight: 'bold', fontStyle: 'italic' }}>Times bold italic, </Text>
          <Text style={{ fontFamily: 'Courier' }}>Courier</Text> and a family list{' '}
          <Text style={{ fontFamily: ['Courier-Bold', 'Lato'] }}>where the first face wins</Text>.
        </Text>

        <Text style={s.label}>sizes 6 to 20</Text>
        <Text>
          {[6, 8, 10, 12, 14, 16, 20].map((n) => (
            <Text key={n} style={{ fontSize: n }}>{`${n}pt `}</Text>
          ))}
        </Text>

        <View style={{ flexDirection: 'row' }}>
          <View style={s.col}>
            <Sample label="left, lineHeight 1" style={{ lineHeight: 1 }} />
            <Sample label="center, lineHeight 16 (pt)" style={{ textAlign: 'center', lineHeight: '16pt' }} />
          </View>
          <View style={s.col}>
            <Sample label="right, lineHeight 150%" style={{ textAlign: 'right', lineHeight: '150%' }} />
            <Sample label="justify" style={{ textAlign: 'justify' }} />
          </View>
          <View style={s.col}>
            <Sample label="letterSpacing 1.5" style={{ letterSpacing: 1.5 }} />
            <Sample label="wordSpacing 6, textIndent 20" style={{ wordSpacing: 6, textIndent: 20 }} />
          </View>
        </View>

        <Text style={s.label}>textTransform, textDecoration</Text>
        <Text>
          <Text style={{ textTransform: 'uppercase' }}>upper case </Text>
          <Text style={{ textTransform: 'lowercase' }}>LOWER CASE </Text>
          <Text style={{ textTransform: 'capitalize' }}>capitalised words here </Text>
          <Text style={{ textDecoration: 'underline' }}>underline</Text>{' '}
          <Text style={{ textDecoration: 'line-through', textDecorationColor: 'red' }}>struck in red</Text>{' '}
          <Text style={{ textDecoration: 'underline line-through', textDecorationStyle: 'dashed' }}>both, dashed</Text>
        </Text>

        <Text style={s.label}>nested runs with inherited styles, links, sub and super, colour and background</Text>
        <Text style={{ color: '#1d3557' }}>
          Water is H<Text style={{ verticalAlign: 'sub', fontSize: 7 }}>2</Text>O and E = mc
          <Text style={{ verticalAlign: 'super', fontSize: 7 }}>2</Text>. Runs can{' '}
          <Text style={{ color: '#e63946', fontWeight: 'bold' }}>
            change colour and <Text style={{ fontStyle: 'italic', backgroundColor: '#ffe066' }}>nest again</Text>
          </Text>{' '}
          and hold a <Link src="https://example.com/docs">link that wraps onto the next line if the line is full</Link>. Emoji 😀 sit in the line.
        </Text>

        <Text style={s.label}>textOverflow ellipsis with maxLines 2</Text>
        <Text style={{ width: 300, maxLines: 2, textOverflow: 'ellipsis' }}>{TEXT + ' ' + TEXT}</Text>

        <View style={{ flexDirection: 'row' }}>
          <View style={[s.col, { width: 110 }]}>
            <Sample label="hyphenation, default" style={{ fontSize: 12 }} />
          </View>
          <View style={[s.col, { width: 110 }]}>
            <Sample label="soft hyphens only" style={{ fontSize: 12 }}>
              {'Extra­ordinarily un­character­istic re­sponsi­bilities of in­ter­national­isation.'}
            </Sample>
          </View>
          <View style={[s.col, { width: 110 }]}>
            <Text style={s.label}>callback that never splits</Text>
            <Text style={{ fontSize: 12 }} hyphenationCallback={(word: string) => [word]}>{TEXT}</Text>
          </View>
          <View style={[s.col, { width: 118, marginRight: 0 }]}>
            <Text style={s.label}>direction rtl, and Hebrew inside left to right</Text>
            <Text style={{ direction: 'rtl', fontFamily: ['Noto Sans Arabic', 'Lato'] }}>
              مرحبا بالعالم، هذا نص عربي (بين قوسين) فيه كلمة PDF والرقم 2026 ويلتف على عدة أسطر.
            </Text>
            <Text style={{ direction: 'rtl', fontFamily: ['Noto Sans Hebrew', 'Lato'], marginTop: 4 }}>
              שלום עולם, זהו טקסט בעברית [עם סוגריים] ומספר 42 באמצע.
            </Text>
            <Text style={{ fontFamily: ['Lato', 'Noto Sans Hebrew'], marginTop: 4 }}>
              Left to right with שלום עולם (inside) and on.
            </Text>
          </View>
        </View>
      </Page>

      <Page size="A5" style={s.page}>
        <Text style={{ fontSize: 16, fontWeight: 'bold', marginBottom: 6 }}>Page two, A5</Text>
        {Array.from({ length: 4 }, (_, i) => (
          <Text key={i} style={{ marginBottom: 6, textAlign: i % 2 ? 'justify' : 'left', fontStyle: i === 3 ? 'italic' : 'normal' }}>
            {TEXT} {TEXT}
          </Text>
        ))}
      </Page>
    </Document>
  );
}
