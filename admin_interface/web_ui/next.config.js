/** @type {import('next').NextConfig} */
const nextConfig = {
  // output: 'export',
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://127.0.0.1:8080/api/:path*' // Proxy to Backend
      }
    ]
  }
};

module.exports = nextConfig;
