import { BarChart, Bar, XAxis, YAxis, ResponsiveContainer, ReferenceLine } from 'recharts';

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
                    top: 60,
                    right: 30,
                    left: 20,
                    bottom: 5,
                }}
            >
                <XAxis dataKey="label" stroke="blue" />
                <YAxis domain={[0, 12]} stroke="blue" label={{ value: 'blocks', fill: 'red' }}tickCount={13}/>
                <Bar dataKey="value" fill="#8a70e7" label={{ fill: 'red', fontSize: 20 }}/>
                <ReferenceLine y={6} stroke="darkred" strokeWidth={4} isFront/>
            </BarChart>

        </ResponsiveContainer>
    )
}
