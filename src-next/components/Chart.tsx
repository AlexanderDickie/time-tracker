import React, { useState } from "react";
import { LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
} from 'recharts';

const Chart = ({data}) => {
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
export default Chart;
