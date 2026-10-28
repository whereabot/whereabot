import DefaultTheme from 'vitepress/theme'
// Tokens only: the design system's element defaults would fight the VitePress layout.
import '../../../design-system/color-tokens.css'
import '../../../design-system/typography-tokens.css'
import './design-token-map.css'

export default DefaultTheme
