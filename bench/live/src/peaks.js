// Each engine's worker memory peak (MB) from our measured runs, by layout and the page count asked
// for, read through DevTools (CDP Runtime.getHeapUsage). A rerun of the matrix updates them here.
export const RESULTS_URL = 'https://github.com/axonpack/axonpdf/blob/main/bench/RESULTS.md';

export const MEASURED = {
  sections: { axonpdf: {}, 'react-pdf': {} },
  flowing: {
    axonpdf: { 100: 6.3, 500: 51.7 },
    'react-pdf': { 100: 530 },
  },
};
