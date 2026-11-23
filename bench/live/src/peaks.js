// Each engine's worker memory peak (MB) from our measured runs, by layout and the page count asked
// for, read through DevTools (CDP Runtime.getHeapUsage). A rerun of the matrix updates them here.
export const RESULTS_URL = 'https://github.com/axonpack/axonpdf/blob/main/bench/RESULTS.md';

export const MEASURED = {
  sections: {
    axonpdf: { 100: 11.6, 500: 55.9, 2000: 141.6 },
    'react-pdf': { 100: 192.5, 500: 2229.9 },
  },
  flowing: {
    // 100 is from an earlier run: the 2026-11-23 matrix run finished before the first sample.
    axonpdf: { 100: 6.3, 500: 39.2 },
    'react-pdf': { 100: 560.1 },
  },
};
