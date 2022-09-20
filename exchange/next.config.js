const { i18n } = require('./next-i18next.config')

const moduleExports = {
  i18n,
  typescript: {
    // !! WARN !!
    // Dangerously allow production builds to successfully complete even if
    // your project has type errors.
    // !! WARN !!
    ignoreBuildErrors: true,
  },
  // rules: { 
  //   "no-unused-vars": "off",
  // },
  async redirects() {
    return [
      {
        source: '/market',
        destination: '/',
        permanent: true,
      },
      {
        source: '/spot/:name',
        destination: '/',
        permanent: true,
      },
      {
        source: '/perp/:name',
        destination: '/',
        permanent: true,
      },
    ]
  },
  webpack: (config, options) => {
    // Important: return the modified config
    if (!options.isServer) {
      config.resolve.fallback.fs = false
    }

    config.module.rules.push({
      test: /\.svg$/,
      use: ['@svgr/webpack'],
    })

    if (process.env.ANALYZE === 'true') {
      const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer')
      config.plugins.push(
        new BundleAnalyzerPlugin({
          analyzerMode: 'static',
          reportFilename: options.isServer
            ? './analyze/server.html'
            : './analyze/client.html',
        })
      )
    }

    return config
  },
}

module.exports = moduleExports
