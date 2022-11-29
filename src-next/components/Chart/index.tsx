import { BarChart, Bar, Cell, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

export type ChartInput = {
    label: String,
    value: number,
}[];

export default function Chart({data}: any) {
    return (
        <ResponsiveContainer width="95%" height="95%">
            <BarChart
                width={500}
                height={300}
                data={data}
                margin={{
                    top: 5,
                    right: 30,
                    left: 20,
                    bottom: 5,
                }}
            >
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="label"  />
                <YAxis />
                <Tooltip />
                <Legend />
                <Bar dataKey="value" fill="#8a70e7" />
                <CartesianGrid strokeDasharray="4 2" />
            </BarChart>

        </ResponsiveContainer>
    )
}
