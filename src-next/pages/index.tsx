import {React, useEffect} from 'react';
import { invoke } from "@tauri-apps/api/tauri"
import Chart from 'components/Chart';

const data = [
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
    useEffect(() => {
        invoke('get_previous_ending_today', { n: 'World' })
            .then(console.log)
            .catch(console.error)
    }, []);

  return (  
    <Chart data = {data} />
  )
}
