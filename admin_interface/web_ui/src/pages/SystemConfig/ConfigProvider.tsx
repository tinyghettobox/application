import {createContext, ReactElement, useContext, useState} from "react";

type ConfigContextValue = {
    config: Record<string, any>,
    setField(name: string, value: unknown, type: 'string' | 'int' | 'float' | 'boolean'): void;
}

const ConfigContext = createContext<ConfigContextValue | undefined>(undefined);

interface Props {
    children: ReactElement | ReactElement[]
}

export default function ConfigProvider(props: Props) {
    const [config, setConfig] = useState({
        sleepTimer: 60
    });

    const contextValue = {
        config,
        setField(name: string, value: unknown, type: 'string' | 'int' | 'float' | 'boolean' = 'string') {
            let parsedValue = value;

            if (type === 'int' && typeof value === 'string') {
                parsedValue = parseInt(value);
            }
            if (type === 'float' && typeof value === 'string') {
                parsedValue = parseFloat(value.replaceAll(',', '.'));
            }

            setConfig(current => ({...current, [name]: parsedValue}));
        }
    };

    return (
        <ConfigContext.Provider value={contextValue}>
            {props.children}
        </ConfigContext.Provider>
    )
}

export function useConfig(): ConfigContextValue {
    const contextValue = useContext(ConfigContext);
    if (!contextValue) {
        throw new Error('ConfigContext not yet set');
    }
    return contextValue;
}