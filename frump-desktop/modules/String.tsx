import S = require("string");

export class String{

    static Trim(value: string): string {
        return value.trim();
    }

    static NormalizeSpace(value: string): string {
        return value;
    }

    static Substring(value: string, start: number, end?: number): string {
        return value.substring(start, end);
    }

    static SubstringBetween(value: string, left: string, right: string): string {
        return S(value).between(left, right).s;
    }

    static Repalce(value: string, oldValue: string, newValue: string): string {
        return S(value).replaceAll(oldValue, newValue).s;
    }
    
    static IndexOf(value: string, search: string): number {
        return value.indexOf(search);
    }
    
    static Split(value: string, separator: string): Array<string> {
        return value.split(separator);
    }
    
    static Contains(value: string, search: string): boolean {
        return value.indexOf(search) >= 0;
    }
}