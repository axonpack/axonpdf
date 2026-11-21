import type { NodeProps } from "./props.type.js";

interface FormCommonProps extends NodeProps {
  name?: string;
  required?: boolean;
  noExport?: boolean;
  readOnly?: boolean;
  value?: number | string;
  defaultValue?: number | string;
}

export interface FieldSetProps extends NodeProps {
  name: string;
}

export interface TextInputFormatting {
  type: "date" | "time" | "percent" | "number" | "zip" | "zipPlus4" | "phone" | "ssn";
  param?: string;
  nDec?: number;
  sepComma?: boolean;
  negStyle?: "MinusBlack" | "Red" | "ParensBlack" | "ParensRed";
  currency?: string;
  currencyPrepend?: boolean;
}

export interface TextInputProps extends FormCommonProps {
  align?: "left" | "center" | "right";
  multiline?: boolean;
  password?: boolean;
  noSpell?: boolean;
  format?: TextInputFormatting;
  fontSize?: number;
  maxLength?: number;
}

export interface CheckboxProps extends FormCommonProps {
  backgroundColor?: string;
  borderColor?: string;
  checked?: boolean;
  onState?: string;
  offState?: string;
  xMark?: boolean;
}

export interface SelectAndListProps extends FormCommonProps {
  sort?: boolean;
  edit?: boolean;
  multiSelect?: boolean;
  noSpell?: boolean;
  select?: string[];
}
