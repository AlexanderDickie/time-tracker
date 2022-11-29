import React, { useEffect, useState } from 'react';
import { invoke } from "@tauri-apps/api/tauri"
import dynamic from "next/dynamic";

import type {ChartInput} from 'components/Chart';
const ChartDynamic = dynamic(
        import('components/Chart'),
        { ssr: false }
    );

const initialData = [
    {
        label: "a",
        value: 5
    },
    {
        label: "b",
        value: 10
    }
];

export default function View() {
    const [state, setState]  = useState<ChartInput>(initialData);

    useEffect(() => {
        invoke<ChartInput>('get_previous')
        .then((response: ChartInput): void  => {
                setState(response);
            })
        .catch(console.error)
    }, []);

    return (
        <div className="responsive">
        <ChartDynamic data = {state} />
        </div>
    );

}

