export default class StrUtil {
  sep(a: string): string {
    return '-' + a
  }

  public addSep(a: string, b: string): string {
    return a + this.sep(this.toStr(b))
  }

  public toStr(a: string): string {
    return a.toString()
  }

  public add(a: string, b: string): string {
    return a + b
  }
}
