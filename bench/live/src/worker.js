import * as axonpdf from '@axonpdf/react';
import { expose } from '@axonpdf/react/worker';
import { createElement } from 'react';
import { LedgerDocument } from './ledger.jsx';

expose((props) => createElement(LedgerDocument, { pdf: axonpdf, ...props }));
