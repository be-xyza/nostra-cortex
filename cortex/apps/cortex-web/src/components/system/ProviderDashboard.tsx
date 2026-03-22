import React, { useEffect } from 'react';
import { useProvidersRegistry } from '../../store/providersRegistry';
import { Globe, Server, Activity, Shield, Cpu, RefreshCw } from 'lucide-react';

export const ProviderDashboard: React.FC = () => {
    const { providers, isLoading, error, fetchProviders } = useProvidersRegistry();

    useEffect(() => {
        fetchProviders();
    }, [fetchProviders]);

    const getIcon = (type: string) => {
        switch (type) {
            case 'Llm': return <Cpu className="w-4 h-4" />;
            case 'Embedding': return <Globe className="w-4 h-4" />;
            case 'Batch': return <Server className="w-4 h-4" />;
            default: return <Activity className="w-4 h-4" />;
        }
    };

    const getLlmBadge = (llmType?: string) => {
        if (!llmType) return null;
        return (
            <span className="chip text-[10px] ml-2 opacity-80 border border-border-strong px-2 py-0.5 rounded-full">
                {llmType}
            </span>
        );
    };

    return (
        <div className="provider-dashboard p-6 animate-in fade-in duration-500">
            <header className="mb-8 flex justify-between items-center">
                <div>
                    <h1 className="text-2xl font-bold tracking-tight text-ink mb-2">System Providers</h1>
                    <p className="text-ink-muted text-sm max-w-2xl">
                        Monitor and manage execution layer API providers. These services power the intelligence and data processing capabilities of Nostra Cortex.
                    </p>
                </div>
                <button 
                    onClick={() => fetchProviders()}
                    disabled={isLoading}
                    className="flex items-center gap-2 px-4 py-2 bg-surface-elevated hover:bg-opacity-80 transition-all rounded-lg text-sm font-semibold border border-border-subtle"
                >
                    <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
                    Refresh
                </button>
            </header>

            {error && (
                <div className="error-banner mb-6 flex items-center gap-3">
                    <Shield className="w-5 h-5 text-bad" />
                    <span>Failed to load provider registry: {error}</span>
                </div>
            )}

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {providers.length === 0 && !isLoading && !error && (
                    <div className="col-span-full py-12 text-center border border-dashed border-border-strong rounded-xl opacity-50">
                        <Activity className="w-12 h-12 mx-auto mb-4 opacity-20" />
                        <p>No providers registered in the current environment configuration.</p>
                    </div>
                )}

                {providers.map((provider) => (
                    <div 
                        key={provider.id} 
                        className="panel group hover:border-accent/30 transition-all duration-300"
                    >
                        <div className="panel-head flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                {getIcon(provider.providerType)}
                                <span>{provider.providerType}</span>
                            </div>
                            <div className={`w-2 h-2 rounded-full ${provider.isActive ? 'bg-ok shadow-[0_0_8px_var(--ok)]' : 'bg-bad opacity-50'}`} />
                        </div>
                        <div className="panel-body">
                            <div className="flex items-baseline justify-between mb-2">
                                <h3 className="text-lg font-semibold text-ink">{provider.name}</h3>
                                {getLlmBadge(provider.llmType)}
                            </div>
                            
                            <div className="space-y-4">
                                <div className="metric">
                                    <span>Endpoint</span>
                                    <code className="text-xs break-all block mt-1 opacity-70">
                                        {provider.endpoint}
                                    </code>
                                </div>

                                {provider.metadata && Object.keys(provider.metadata).length > 0 && (
                                    <div className="metric">
                                        <span>Metadata</span>
                                        <div className="grid grid-cols-1 gap-1 mt-2">
                                            {Object.entries(provider.metadata).map(([key, val]) => (
                                                <div key={key} className="flex justify-between text-[11px]">
                                                    <span className="text-ink-muted lowercase">{key}</span>
                                                    <span className="text-ink opacity-80">{val}</span>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                <div className="flex items-center justify-between pt-2 border-t border-border-subtle">
                                    <span className="text-[10px] uppercase tracking-wider text-ink-faint font-bold">Status</span>
                                    <span className={`text-xs font-bold ${provider.isActive ? 'text-ok' : 'text-bad'}`}>
                                        {provider.isActive ? 'OPERATIONAL' : 'OFFLINE'}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                ))}

                {isLoading && (
                    <>
                        {[1, 2, 3].map((i) => (
                            <div key={i} className="panel opacity-50 animate-pulse h-64">
                                <div className="panel-head h-10 bg-surface-elevated"></div>
                                <div className="panel-body"></div>
                            </div>
                        ))}
                    </>
                )}
            </div>
        </div>
    );
};
