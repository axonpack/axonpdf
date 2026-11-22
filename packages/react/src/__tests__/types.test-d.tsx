// Code as users write it against the components' published types; `check-types` compiles it.
// Nothing here runs.
import ReactPDF, {
  BlobProvider,
  Document,
  Font,
  Image,
  ImageBackground,
  Link,
  Page,
  PDFDownloadLink,
  PDFViewer,
  Path,
  Select,
  StyleSheet,
  Svg,
  Text,
  View,
  pdf,
  renderToBuffer,
  renderToFile,
  renderToStream,
  usePDF,
} from "../index.node.js";
import type {
  BlobProviderParams,
  DocumentProps,
  FormCommonProps,
  ImageBackgroundWithSourceProp,
  ImageWithSrcProp,
  PDFRenderer,
  SelectAndListPropsBase,
  Styles,
} from "../index.node.js";

const styles: ReactPDF.Styles = StyleSheet.create({
  page: { padding: 30, flexDirection: "row" },
  title: { fontSize: 24, fontFamily: "Roboto", fontWeight: "bold" },
});
const named: Styles = styles;

Font.register({
  family: "Roboto",
  fonts: [{ src: "Roboto-Regular.ttf" }, { src: "Roboto-Bold.ttf", fontWeight: 700 }],
});
Font.registerHyphenationCallback((word) => [word]);
Font.registerEmojiSource({ format: "png", url: "https://example.com/emoji/" });

const meta: ReactPDF.DocumentProps = {
  title: "Invoice",
  onRender: ({ blob }) => blob?.size,
};
const same: DocumentProps = meta;

function Invoice(props: ReactPDF.PageProps) {
  return (
    <Page {...props} style={[named.page, { margin: 10 }]}>
      <View style={styles.page} wrap={false} fixed>
        <Text
          style={styles.title}
          render={({ pageNumber, totalPages }) => `${pageNumber}/${totalPages}`}
        />
        <Link src="https://example.com">site</Link>
      </View>
      <Image src={{ uri: "https://example.com/a.png", method: "GET", headers: {}, body: "" }} />
      <Image source={() => Promise.resolve(Buffer.from(""))} cache={false} />
      <ImageBackground src="a.png" imageStyle={{ opacity: 0.5 }}>
        <Text>Over the image</Text>
      </ImageBackground>
      <Svg viewBox="0 0 10 10">
        <Path d="M0 0 L10 10" stroke="red" />
      </Svg>
      <Select name="ward" select={["North", "South"]} edit={false} noSpell />
    </Page>
  );
}

const doc = (
  <Document {...same}>
    <Invoice size="A4" />
  </Document>
);

const image: ReactPDF.ImageWithSrcProp = { src: "a.png", srcSet: "a.png 300w", sizes: 300 };
const background: ImageBackgroundWithSourceProp = { source: "b.png", imageStyle: {} };
const field: FormCommonProps = { name: "x", required: true, value: 1 };
const list: SelectAndListPropsBase = { select: ["a"], multiSelect: true };
const params: BlobProviderParams = { blob: null, url: null, loading: true, error: null };
const renderer = (r: PDFRenderer) => r.createContainer({ document: null });
const sources: ImageWithSrcProp[] = [image];

export async function node() {
  await ReactPDF.render(doc, "out.pdf");
  await renderToFile(doc, "out.pdf", (output, path) => [output, path]);
  const stream: NodeJS.ReadableStream = await renderToStream(doc);
  const buffer: Buffer = await renderToBuffer(doc);
  const instance = pdf(doc);
  instance.on("change", () => {});
  instance.updateContainer(doc, () => {});
  const blob: Blob = await instance.toBlob();
  return [stream, buffer, blob, background, field, list, params, renderer, sources];
}

export function Browser() {
  const [state, update] = usePDF({ document: doc });
  return (
    <>
      <PDFViewer width="100%" height={600} showToolbar={false}>
        {doc}
      </PDFViewer>
      <PDFDownloadLink document={doc} fileName="invoice.pdf" onClick={() => update(doc)}>
        {({ loading }) => (loading ? "Loading" : state.url)}
      </PDFDownloadLink>
      <BlobProvider document={doc}>
        {({ blob, url, error }) => (error ? null : (url ?? blob?.type))}
      </BlobProvider>
    </>
  );
}
