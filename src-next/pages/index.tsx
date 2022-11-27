import React, { useEffect, useState } from 'react';
import Chart  from 'components/Chart';
import { invoke } from "@tauri-apps/api/tauri"

const initialData = [
    {
        name: "a",
        value: 0
    },
    {
        name: "b",
        value: 0
    }
];


type ChartInput = {
    name: string;
    value: number;
}[];

export default function Index() {
    const [state, setState]  = useState<ChartInput>(initialData);

    useEffect(() => {
        invoke<ChartInput>('get_previous')
        .then((response: ChartInput): void  => {
                setState(response);
            })
        .catch(console.error)
    }, []);


    return (
        <Chart chartInput = {state} />
    );
}
