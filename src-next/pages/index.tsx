import React, { useEffect, useState } from 'react';
import Chart from 'components/Chart';
import { invoke } from "@tauri-apps/api/tauri"

const data1 = [
  {
    name: "Sun",
    value: 10
  },
  {
    name: "Mon",
    value: 30
  },
  {
    name: "Tue",
    value: 100
  },
  {
    name: "Wed",
    value: 30
  },
  {
    name: "Thu",
    value: 23
  },
  {
    name: "Fri",
    value: 34
  },
  {
    name: "Sat",
    value: 11
  }
];

export default function Index() {
    const [data,setData] = useState(data1);

    useEffect(() => {
        invoke('get_previous')
        .then((response) => {
                console.log(response);
                setData(response);
            })
        .catch(console.error)
    }, []);


    return (
        <Chart data = {data} />
    );
}
