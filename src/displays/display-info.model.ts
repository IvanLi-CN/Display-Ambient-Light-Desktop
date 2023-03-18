export class DisplayInfo {
  constructor(
    public id: number,
    public x: number,
    public y: number,
    public width: number,
    public height: number,
    public scale_factor: number,
    public is_primary: boolean,
  ) {}
}