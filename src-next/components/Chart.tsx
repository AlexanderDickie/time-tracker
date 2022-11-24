import React, { useState } from "react";
import { LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
} from 'recharts';

// const data = [
//   {
//     name: "Sun",
//     value: 10
//   },
//   {
//     name: "Mon",
//     value: 30
//   },
//   {
//     name: "Tue",
//     value: 100
//   },
//   {
//     name: "Wed",
//     value: 30
//   },
//   {
//     name: "Thu",
//     value: 23
//   },
//   {
//     name: "Fri",
//     value: 34
//   },
//   {
//     name: "Sat",
//     value: 11
//   }
// ];
//
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
