/**
 * Shared utility functions for formatting API documentation
 */

export function formatType(type: string): string {
    return type;
}

export function formatParameters(params?: Array<{ name: string; type: string }>): string {
    if (!params || params.length === 0) return '';
    return params.map(p => `${p.name}: ${p.type}`).join(', ');
}

export function formatReturns(returns?: Array<{ type: string }>): string {
    if (!returns || returns.length === 0) return 'void';
    return returns.map(r => r.type).join(', ');
}
