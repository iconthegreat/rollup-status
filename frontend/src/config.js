// API configuration
// In production, set VITE_API_URL to your Railway backend URL
const API_URL = import.meta.env.VITE_API_URL || ''

export const config = {
  apiUrl: API_URL,
  wsUrl: API_URL ? API_URL.replace(/^http/, 'ws') : '',
}
