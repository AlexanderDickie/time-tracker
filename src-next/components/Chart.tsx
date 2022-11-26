import { LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
} from 'recharts';


type ChartInput = {
    name: String,
    value: number,
}

export default function Chart({ data }: {data: Array<ChartInput>}) {
  return (
        <LineChart width={650} height={300} data={data}>
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
