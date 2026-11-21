// The three react-reconciler builds, installed under aliases so each React version gets its own.
declare module "react-reconciler-18" {
  const createReconciler: (config: object) => any;
  export default createReconciler;
}
declare module "react-reconciler-19-0" {
  const createReconciler: (config: object) => any;
  export default createReconciler;
}
declare module "react-reconciler-19-2" {
  const createReconciler: (config: object) => any;
  export default createReconciler;
}
