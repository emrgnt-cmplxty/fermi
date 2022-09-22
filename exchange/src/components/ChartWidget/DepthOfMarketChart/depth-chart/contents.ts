import { curveStepAfter } from 'd3-shape'

import { DepthCurve } from './display-objects'
import { Colors } from './helpers'
import { Renderer } from './renderer'
import { Container } from './renderer/display'

type ContentsColors = Pick<
  Colors,
  'bidFill' | 'bidStroke' | 'askFill' | 'askStroke'
>

/**
 * Responsible for drawing area curves for depth chart.
 */
export class Contents {
  public stage: Container = new Container()
  public renderer: Renderer

  public buyCurve: DepthCurve
  public sellCurve: DepthCurve

  public colors: ContentsColors

  constructor(options: {
    view: HTMLCanvasElement
    resolution: number
    width: number
    height: number
    colors: ContentsColors
  }) {
    this.renderer = new Renderer({
      view: options.view,
      resolution: options.resolution,
      width: options.width,
      height: options.height,
    })

    this.colors = options.colors

    this.buyCurve = new DepthCurve(
      options.colors.bidStroke,
      options.colors.bidFill,
      curveStepAfter,
    )

    this.sellCurve = new DepthCurve(
      options.colors.askStroke,
      options.colors.askFill,
      curveStepAfter,
    )

    this.stage.addChild(this.buyCurve)
    this.stage.addChild(this.sellCurve)
  }

  public render(): void {
    this.renderer.render(this.stage)
  }

  public update(
    buyPoints: [number, number][],
    sellPoints: [number, number][],
  ): void {
    const resolution = this.renderer.resolution

    this.buyCurve.update(
      buyPoints,
      this.renderer.view.height,
      resolution,
      this.colors.bidFill,
      this.colors.bidStroke,
    )

    this.sellCurve.update(
      sellPoints,
      this.renderer.view.height,
      resolution,
      this.colors.askFill,
      this.colors.askStroke,
    )
  }
}
