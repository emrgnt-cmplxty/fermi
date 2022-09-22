import { extent, max, mean, min } from 'd3-array'
import { scaleLinear } from 'd3-scale'
import EventEmitter from 'eventemitter3'
import { orderBy, sortBy, zip } from 'lodash'

import { Contents } from './contents'
import { AXIS_HEIGHT, PriceLevel } from './depth-chart'
import { Colors } from './helpers'
import cumsum from './math/array/cumsum'
import { UI } from './ui'

function getMidPrice(
  indicativePrice: number,
  midPrice: number,
  bidPrice: number,
  askPrice: number,
): number {
  if (indicativePrice) {
    return indicativePrice
  }

  if (midPrice) {
    return midPrice
  }

  return mean([bidPrice, askPrice]) as number
}

export const sizeFormatter = new Intl.NumberFormat('en-gb', {
  maximumFractionDigits: 0,
  minimumFractionDigits: 0,
})

export class Chart extends EventEmitter {
  private chart: Contents
  private axis: UI

  private prices: number[] = []
  private sizes: number[] = []
  private priceLabels: string[] = []
  private sizeLabels: string[] = []

  private _span = 1
  private maxPriceDifference = 0

  private _data: { bids: PriceLevel[]; asks: PriceLevel[] } = {
    bids: [],
    asks: [],
  }

  /** Indicative price if the auction ended now, 0 if not in auction mode */
  private _indicativePrice = 0

  /** Arithmetic average of the best bid price and best offer price. */
  private _midPrice = 0

  private priceFormat: (price: number) => string

  private _colors: Colors

  constructor(options: {
    chartView: HTMLCanvasElement
    axisView: HTMLCanvasElement
    resolution: number
    width: number
    height: number
    priceFormat: (price: number) => string
    colors: Colors
  }) {
    super()

    this.priceFormat = options.priceFormat
    this._colors = options.colors

    this.chart = new Contents({
      view: options.chartView,
      resolution: options.resolution,
      width: options.width,
      height: options.height,
      colors: options.colors,
    })

    this.axis = new UI({
      view: options.axisView,
      resolution: options.resolution,
      width: options.width,
      height: options.height,
      colors: options.colors,
    })

    this.axis
      .on('zoomstart', () => {
        this.emit('zoomstart')
      })
      .on('zoom', (k: number) => {
        this.span = 1 / k
        this.emit('zoom')
      })
      .on('zoomend', () => {
        this.emit('zoomend')
      })
  }

  public updatePrice(price: number) {
    this.axis.updatePrice(price)
  }

  public clearPrice() {
    this.axis.clearPrice()
  }

  public render() {
    this.chart.render()
    this.axis.render()
  }

  public resize(width: number, height: number) {
    this.chart.renderer.resize(width, height)
    this.axis.renderer.resize(width, height)
  }

  public destroy() {
    this.axis.destroy()
  }

  private update() {
    const resolution = this.axis.renderer.resolution

    const cumulativeBuy = zip<number>(
      this._data.bids.map((priceLevel) => priceLevel.price),
      cumsum(this._data.bids.map((priceLevel) => priceLevel.size)),
    ) as [number, number][]

    const cumulativeSell = zip<number>(
      this._data.asks.map((priceLevel) => priceLevel.price),
      cumsum(this._data.asks.map((priceLevel) => priceLevel.size)),
    ) as [number, number][]

    const midPrice = getMidPrice(
      this._indicativePrice,
      this._midPrice,
      this._data.bids?.[0]?.price,
      this._data.asks?.[0]?.price,
    )

    if (!this.maxPriceDifference) {
      this.maxPriceDifference =
        max(this.prices.map((price) => Math.abs(price - midPrice))) ?? 0
    }

    const priceExtent: [number, number] = [
      midPrice - this._span * this.maxPriceDifference,
      midPrice + this._span * this.maxPriceDifference,
    ]

    const indexExtent = extent(
      orderBy([...this._data.bids, ...this._data.asks], ['price'])
        .map((priceLevel, index) => ({ ...priceLevel, index }))
        .filter(
          (priceLevel) =>
            priceLevel.price >= priceExtent[0] &&
            priceLevel.price <= priceExtent[1],
        )
        .map((priceLevel) => priceLevel.index),
    )

    const sizeExtent: [number, number] = [
      0,
      2 * (max(this.sizes.slice(indexExtent[0], indexExtent[1])) ?? 0),
    ]

    const priceScale = scaleLinear().domain(priceExtent).range([0, this.width])

    const sizeScale = scaleLinear()
      .domain(sizeExtent)
      .range([this.height - resolution * AXIS_HEIGHT, 0])

    // Add dummy data points at extreme points of price range
    // to ensure the chart looks symmetric
    if (cumulativeBuy.length > 0) {
      cumulativeBuy.push([
        midPrice - this.maxPriceDifference,
        cumulativeBuy[cumulativeBuy.length - 1][1],
      ])
    }

    if (cumulativeSell.length > 0) {
      cumulativeSell.push([
        midPrice + this.maxPriceDifference,
        cumulativeSell[cumulativeSell.length - 1][1],
      ])
    }

    this.chart.colors = this._colors

    this.chart.update(
      cumulativeBuy.map((point) => [priceScale(point[0]), sizeScale(point[1])]),
      cumulativeSell.map((point) => [
        priceScale(point[0]),
        sizeScale(point[1]),
      ]),
    )

    // TODO: Clean up this logic
    if (this._data.bids.length > 0 && this._data.asks.length > 0) {
      const minExtent =
        (min(
          this.prices
            .filter((price) => midPrice - price > 0)
            .map((price) => midPrice - price),
        ) as number) ??
        0 +
          (min(
            this.prices
              .filter((price) => price - midPrice > 0)
              .map((price) => price - midPrice),
          ) as number) ??
        0

      this.axis.scaleExtent = [
        1,
        this.maxPriceDifference /
          (2 * (minExtent ?? this.maxPriceDifference / 10)),
      ]
    }

    this.axis.colors = this._colors

    this.axis.update(
      this.prices.map((price) => priceScale(price)),
      this.sizes.map((size) => sizeScale(size)),
      midPrice,
      this.priceLabels,
      this.sizeLabels,
      this.priceFormat(midPrice),
      this._indicativePrice > 0 ? 'Indicative price' : 'Mid Market Price',
      priceScale,
      sizeScale,
    )
  }

  set colors(colors: Colors) {
    this._colors = colors

    this.update()
    this.render()
  }

  get data() {
    return this._data
  }

  set data(data: { bids: PriceLevel[]; asks: PriceLevel[] }) {
    this._data = data

    this.prices = sortBy([
      ...this._data.bids.map((priceLevel) => priceLevel.price),
      ...this._data.asks.map((priceLevel) => priceLevel.price),
    ])

    this.priceLabels = this.prices.map((price) => this.priceFormat(price))

    const cumulativeBuy = zip<number>(
      this._data.bids.map((priceLevel) => priceLevel.price),
      cumsum(this._data.bids.map((priceLevel) => priceLevel.size)),
    ) as [number, number][]

    const cumulativeSell = zip<number>(
      this._data.asks.map((priceLevel) => priceLevel.price),
      cumsum(this._data.asks.map((priceLevel) => priceLevel.size)),
    ) as [number, number][]

    this.sizes = orderBy([...cumulativeBuy, ...cumulativeSell], ['0']).map(
      (priceLevel) => priceLevel[1],
    )

    this.sizeLabels = this.sizes.map((size) => sizeFormatter.format(size))

    this.update()
    this.render()
  }

  set indicativePrice(price: number) {
    this._indicativePrice = price

    this.axis.indicativePrice = price

    this.update()
    this.render()
  }

  set midPrice(price: number) {
    this._midPrice = price

    this.update()
    this.render()
  }

  get height(): number {
    return this.chart.renderer.view.height
  }

  get width(): number {
    return this.chart.renderer.view.width
  }

  get span() {
    return this._span
  }

  set span(span: number) {
    this._span = span

    this.update()
    this.render()
  }
}
