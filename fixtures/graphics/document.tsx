// Two pages of drawing: the SVG set (shapes, paths, groups, gradients, clip paths, markers, text),
// Canvas paint, PNG with alpha and JPEG images with fit and position, ImageBackground, a Note,
// then the form fields.
import { readFileSync } from 'node:fs';
import {
  Canvas, Checkbox, Circle, ClipPath, Defs, Document, Ellipse, FieldSet, G, Image, ImageBackground,
  Line, LinearGradient, List, Marker, Note, Page, Path, Polygon, Polyline, RadialGradient, Rect,
  Select, StyleSheet, Stop, Svg, Text, TextInput, Tspan, View,
} from '@axonpdf/react';

const s = StyleSheet.create({
  page: { padding: 30, fontSize: 9 },
  label: { fontSize: 7, color: '#888', marginTop: 10, marginBottom: 3 },
  row: { flexDirection: 'row', alignItems: 'flex-start', gap: 10 },
  frame: { width: 110, height: 80, backgroundColor: '#ffe8d6', border: '0.5pt solid #999' },
  field: { flexDirection: 'row', alignItems: 'center', marginBottom: 8 },
  fieldLabel: { width: 90 },
});

const FITS = ['fill', 'contain', 'cover', 'none', 'scale-down'] as const;

export default function Graphics() {
  return (
    <Document title="Graphics and forms">
      <Page size="A4" style={s.page}>
        <Text style={s.label}>Svg: shapes, paths, groups with transforms, gradients, clip path, markers, text</Text>
        <Svg width={535} height={180} viewBox="0 0 535 180" style={{ backgroundColor: '#f8f9fa' }}>
          <Defs>
            <LinearGradient id="lin" x1="0" y1="0" x2="1" y2="0">
              <Stop offset="0" stopColor="#3a86ff" />
              <Stop offset="1" stopColor="#ff006e" stopOpacity={0.6} />
            </LinearGradient>
            <RadialGradient id="rad" cx="0.5" cy="0.5" r="0.5" fx="0.3" fy="0.3">
              <Stop offset="0%" stopColor="white" />
              <Stop offset="100%" stopColor="#8338ec" />
            </RadialGradient>
            <ClipPath id="clip">
              <Circle cx="460" cy="60" r="40" />
            </ClipPath>
            <Marker id="arrow" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto">
              <Path d="M 0 0 L 10 5 L 0 10 z" fill="#333" />
            </Marker>
          </Defs>
          <Rect x="10" y="10" width="90" height="60" rx="8" fill="url(#lin)" stroke="#222" strokeWidth={1.5} />
          <Circle cx="150" cy="40" r="30" fill="url(#rad)" />
          <Ellipse cx="230" cy="40" rx="40" ry="22" fill="#ffbe0b" fillOpacity={0.7} stroke="#fb5607" strokeDasharray="4, 2" />
          <Line x1="280" y1="15" x2="340" y2="65" stroke="#06d6a0" strokeWidth={6} strokeLinecap="round" />
          <Polyline points="350,65 365,20 380,55 395,15 410,65" fill="none" stroke="#118ab2" strokeWidth={3} strokeLinejoin="round" />
          <G clipPath="url(#clip)">
            <Rect x="420" y="20" width="80" height="80" fill="#ef476f" />
            <Rect x="460" y="20" width="40" height="40" fill="#073b4c" />
          </G>
          <G transform="translate(20, 100) rotate(-8)" opacity={0.85}>
            <Polygon points="0,60 30,0 60,60" fill="#8ac926" stroke="#1a1a1a" />
            <Path d="M 80 60 C 80 0, 140 0, 140 60 S 200 120, 200 60" fill="none" stroke="#6a4c93" strokeWidth={2} />
            <Path d="M 220 10 h 50 v 50 h -50 z M 232 22 h 26 v 26 h -26 z" fill="#ff595e" fillRule="evenodd" />
          </G>
          <Line x1="300" y1="150" x2="380" y2="110" stroke="#333" strokeWidth={1.5} markerEnd="url(#arrow)" />
          <Text x="400" y="140" fill="#1d3557" style={{ fontSize: 14 }} textAnchor="start">
            Svg text <Tspan fill="#e63946" style={{ fontFamily: 'Helvetica-Bold' }}>tspan</Tspan>
          </Text>
          <Text x="470" y="165" textAnchor="middle" style={{ fontSize: 9 }} fill="#666">anchored middle</Text>
        </Svg>

        <Text style={s.label}>Svg preserveAspectRatio: the same 2:1 viewBox in a square, meet and slice</Text>
        <View style={s.row}>
          {['xMidYMid meet', 'xMinYMin meet', 'xMidYMid slice', 'none'].map((p) => (
            <Svg key={p} width={60} height={60} viewBox="0 0 100 50" preserveAspectRatio={p} style={{ border: '0.5pt solid #999' }}>
              <Rect x="0" y="0" width="100" height="50" fill="#a8dadc" />
              <Circle cx="25" cy="25" r="20" fill="#e63946" />
            </Svg>
          ))}
        </View>

        <Text style={s.label}>Canvas paint</Text>
        <Canvas
          style={{ width: 535, height: 90, backgroundColor: '#fdfcdc' }}
          paint={(p: any, w: number, h: number) => {
            p.save().rect(10, 10, 80, 60).fillColor('#00afb9').fill();
            p.roundedRect(100, 10, 80, 60, 12).lineWidth(3).strokeColor('#f07167').stroke();
            p.circle(230, 40, 30).fillOpacity(0.5).fillColor('#0081a7').fill().fillOpacity(1);
            p.save().moveTo(280, 70).bezierCurveTo(300, 0, 340, 0, 360, 70).quadraticCurveTo(380, 30, 400, 70).lineWidth(2).dash(5, { space: 3 }).stroke().restore();
            p.polygon([420, 70], [450, 10], [480, 70]).fillAndStroke('#fed9b7', '#333');
            p.restore().fontSize(10).fillColor('#333').text(`paint got ${Math.round(w)} x ${Math.round(h)}`, 10, h - 14);
            return null;
          }}
        />

        <Text style={s.label}>Image: PNG with alpha over colour, JPEG with each objectFit, objectPosition, Buffer and data sources, an SVG file</Text>
        <View style={s.row}>
          <View style={{ backgroundColor: '#ffd166', padding: 6 }}>
            <Image src="assets/badge.png" style={{ width: 70, height: 70 }} />
          </View>
          {FITS.map((fit) => (
            <View key={fit}>
              <Image src="assets/photo.jpg" style={{ width: 70, height: 50, objectFit: fit, backgroundColor: '#ddd' }} />
              <Text style={{ fontSize: 6 }}>{fit}</Text>
            </View>
          ))}
        </View>
        <View style={[s.row, { marginTop: 6 }]}>
          <Image src="assets/photo.jpg" style={{ width: 90, height: 50, objectFit: 'contain', objectPosition: 'left top', backgroundColor: '#ddd' }} />
          <Image src={readFileSync('assets/badge.png')} style={{ width: 40, borderRadius: 20, backgroundColor: '#264653' }} />
          <Image src={{ data: readFileSync('assets/photo.jpg'), format: 'jpg' }} style={{ height: 50, opacity: 0.5 }} />
          <Image src="assets/logo.svg" style={{ width: 100, height: 50 }} />
        </View>

        <Text style={s.label}>ImageBackground with imageStyle and children, and a Note</Text>
        <View style={s.row}>
          <ImageBackground src="assets/photo.jpg" style={{ width: 220, height: 90, padding: 10, justifyContent: 'flex-end' }} imageStyle={{ opacity: 0.6, objectFit: 'cover' }}>
            <Text style={{ color: 'white', fontSize: 14, fontFamily: 'Helvetica-Bold' }}>Text over a photo</Text>
          </ImageBackground>
          <Note style={{ color: '#ffbe0b' }}>Reviewed on 21 November</Note>
        </View>
      </Page>

      <Page size="A5" style={s.page}>
        <Text style={{ fontSize: 14, fontFamily: 'Helvetica-Bold', marginBottom: 10 }}>Form fields</Text>
        <FieldSet name="resident">
          <View style={s.field}>
            <Text style={s.fieldLabel}>Name</Text>
            <TextInput name="name" value="Ada Lovelace" style={{ width: 200, height: 18 }} fontSize={10} maxLength={40} required />
          </View>
          <View style={s.field}>
            <Text style={s.fieldLabel}>Notes</Text>
            <TextInput name="notes" multiline align="left" style={{ width: 200, height: 50 }} defaultValue="Multi-line text" />
          </View>
          <View style={s.field}>
            <Text style={s.fieldLabel}>PIN</Text>
            <TextInput name="pin" password noSpell style={{ width: 80, height: 18 }} />
          </View>
          <View style={s.field}>
            <Text style={s.fieldLabel}>Date</Text>
            <TextInput name="date" format={{ type: 'date', param: 'dd/mm/yyyy' }} style={{ width: 100, height: 18 }} />
          </View>
          <View style={s.field}>
            <Text style={s.fieldLabel}>Consent</Text>
            <Checkbox name="consent" checked backgroundColor="#e0e0e0" borderColor="#333" style={{ width: 14, height: 14 }} />
            <Text style={{ marginLeft: 20 }}>X mark</Text>
            <Checkbox name="xmark" xMark checked={false} onState="Yes" offState="Off" style={{ width: 14, height: 14, marginLeft: 6 }} />
          </View>
          <View style={s.field}>
            <Text style={s.fieldLabel}>Ward</Text>
            <Select name="ward" select={['North', 'South', 'East']} value="South" edit={false} noSpell style={{ width: 120, height: 18 }} />
          </View>
          <View style={s.field}>
            <Text style={s.fieldLabel}>Allergies</Text>
            <List name="allergies" select={['None', 'Penicillin', 'Latex', 'Nuts']} multiSelect sort edit={false} noSpell style={{ width: 120, height: 60 }} />
          </View>
          <View style={s.field}>
            <Text style={s.fieldLabel}>Signed by</Text>
            <TextInput name="signed" readOnly noExport value="Read only" style={{ width: 120, height: 18 }} />
          </View>
        </FieldSet>
      </Page>
    </Document>
  );
}
