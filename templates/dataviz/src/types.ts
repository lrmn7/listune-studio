export interface DataItem {
  label: string;
  value: number;
  delta?: string;
  highlight?: boolean;
}

export interface DatavizProps {
  title: string;
  headlineMetric: string;
  metricLabel: string;
  sourceNote?: string;
  data: DataItem[];
}
