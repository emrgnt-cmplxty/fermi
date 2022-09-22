import { string2hex } from '../renderer/utils'

export interface Colors {
  bidFill: number
  bidStroke: number
  askFill: number
  askStroke: number
  backgroundSurface: number
  textPrimary: number
  textSecondary: number
}

export function getColors(element: HTMLElement | null): Colors {
  const cssStyleDeclaration = element ? getComputedStyle(element) : null

  return {
    bidFill: string2hex(
      cssStyleDeclaration
        ?.getPropertyValue('--pennant-color-buy-fill')
        .trim() || '#16452d',
    ),
    bidStroke: string2hex(
      cssStyleDeclaration
        ?.getPropertyValue('--pennant-color-buy-stroke')
        .trim() || '#26ff8a',
    ),
    askFill: string2hex(
      cssStyleDeclaration
        ?.getPropertyValue('--pennant-color-sell-fill')
        .trim() || '#800700',
    ),
    askStroke: string2hex(
      cssStyleDeclaration
        ?.getPropertyValue('--pennant-color-sell-stroke')
        .trim() || '#ff261a',
    ),
    textPrimary: string2hex(
      cssStyleDeclaration
        ?.getPropertyValue('--pennant-font-color-base')
        .trim() || '#ffffff',
    ),
    textSecondary: string2hex(
      cssStyleDeclaration
        ?.getPropertyValue('--pennant-font-color-secondary')
        .trim() || '#fafafa',
    ),
    backgroundSurface: string2hex(
      cssStyleDeclaration
        ?.getPropertyValue('--pennant-background-surface-color')
        .trim() || '#0a0a0a',
    ),
  }
}
