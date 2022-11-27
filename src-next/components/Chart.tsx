import { LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
} from 'recharts';


export type ChartInput = {
    name: String,
    value: number,
}[];

export default function Chart({chartInput}: any) {
  return (
        <LineChart width={650} height={300} data={chartInput}>
            <CartesianGrid vertical={false} opacity="0.2" />
            <XAxis
                />
            <YAxis
                />
            <Line
                dataKey="value"
                />
        </LineChart>
        );
};
