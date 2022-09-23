import { CurveFactory, curveStepBefore } from 'd3-shape'

import { AXIS_HEIGHT } from '../depth-chart'
import { Container } from '../renderer/display'
import { Graphics } from '../renderer/graphics'

/**
 * Draws two area curves
 */
export class DepthCurve extends Container {
  private area: Graphics = new Graphics()
  private line: Graphics = new Graphics()

  private stroke: number
  private fill: number
  private curve: CurveFactory

  constructor(
    stroke = 0,
    fill = 0xffffff,
    curve: CurveFactory = curveStepBefore,
  ) {
    super()

    this.stroke = stroke
    this.fill = fill
    this.curve = curve

    this.area.lineStyle({ width: 0 })
    this.line.lineStyle({ width: 1, color: stroke, alpha: 0.5 })

    this.addChild(this.area)
    this.addChild(this.line)
  }

  public update(
    points: [number, number][],
    height: number,
    resolution = 1,
    fill = 0xffffff,
    stroke = 0,
  ): void {
    this.fill = fill
    this.stroke = stroke

    this.area.clear()
    this.area.beginFill(this.fill, 1)
    this.area.drawArea(points, this.curve, height - resolution * AXIS_HEIGHT)
    this.area.endFill()

    this.line.clear()
    this.line.lineStyle({ width: 1, color: this.stroke })
    this.line.drawLine(points, this.curve)
  }
}
