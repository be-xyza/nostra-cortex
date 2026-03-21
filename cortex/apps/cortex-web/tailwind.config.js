export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            colors: {
                'cortex-bg': 'var(--bg)',
                'cortex-bg-panel': 'var(--bg-panel)',
                'cortex-bg-elev': 'var(--bg-elev)',
                'cortex-surface-base': 'var(--surface-base)',
                'cortex-surface-panel': 'var(--surface-panel)',
                'cortex-surface-elevated': 'var(--surface-elevated)',
                'cortex-border-subtle': 'var(--border-subtle)',
                'cortex-border-strong': 'var(--border-strong)',
                'cortex-ink': 'var(--ink)',
                'cortex-ink-muted': 'var(--ink-muted)',
                'cortex-ink-faint': 'var(--ink-faint)',
                'cortex-ok': 'var(--ok)',
                'cortex-warn': 'var(--warn)',
                'cortex-bad': 'var(--bad)',
                'cortex-line': 'var(--line)',
                'cortex-accent': '#3B82F6', // Sync with mock cortex-accent
                'cortex': {
                    950: '#070B14',
                    900: '#0B1221',
                    800: '#15213A',
                    700: '#233454',
                    600: '#344A73',
                    500: '#4A6596',
                }
            },
            borderRadius: {
                'cortex': 'var(--radius)',
            },
            boxShadow: {
                'cortex': 'var(--shadow)',
            },
            fontFamily: {
                sans: ['Inter', 'Avenir Next', 'Segoe UI', 'sans-serif'],
                mono: ['JetBrains Mono', 'monospace'],
            }
        },
    },
    plugins: [],
}
