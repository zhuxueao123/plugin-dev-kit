import { computed } from 'vue'
import zhCn from '../../../locales/zh-CN.json'
import enUs from '../../../locales/en-US.json'

const catalogs = {
  'zh-CN': zhCn,
  'en-US': enUs
}

function normalizeLocale(locale) {
  const raw = String(locale || '').toLowerCase()
  if (raw.startsWith('zh')) {
    return 'zh-CN'
  }
  return 'en-US'
}

function resolvePath(target, path) {
  const parts = String(path || '')
    .split('.')
    .map((part) => part.trim())
    .filter(Boolean)

  let current = target
  for (const part of parts) {
    if (!current || typeof current !== 'object' || !(part in current)) {
      return undefined
    }
    current = current[part]
  }

  return current
}

function formatMessage(template, params) {
  if (!params || typeof template !== 'string') {
    return template
  }

  return template.replace(/\{([^}]+)\}/g, (_, key) => {
    const trimmed = String(key || '').trim()
    return trimmed && trimmed in params ? String(params[trimmed]) : `{${trimmed}}`
  })
}

export function usePluginI18n(pluginCode) {
  const locale = computed(() => normalizeLocale(globalThis?.navigator?.language))

  function normalizeKey(key) {
    const rawKey = String(key || '').trim()
    if (!rawKey) {
      return ''
    }
    const prefix = `${pluginCode}.`
    return rawKey.startsWith(prefix) ? rawKey : `${prefix}${rawKey}`
  }

  function translate(key, params, defaultValue = '') {
    const normalizedKey = normalizeKey(key)
    if (!normalizedKey) {
      return defaultValue
    }

    const activeCatalog = catalogs[locale.value] || catalogs['en-US']
    const fallbackCatalog = catalogs['en-US']
    const value = resolvePath(activeCatalog, normalizedKey) ?? resolvePath(fallbackCatalog, normalizedKey)
    if (typeof value === 'string') {
      return formatMessage(value, params)
    }
    return defaultValue || normalizedKey
  }

  function exists(key) {
    const normalizedKey = normalizeKey(key)
    if (!normalizedKey) {
      return false
    }
    const activeCatalog = catalogs[locale.value] || catalogs['en-US']
    return resolvePath(activeCatalog, normalizedKey) !== undefined || resolvePath(catalogs['en-US'], normalizedKey) !== undefined
  }

  return {
    locale,
    t: translate,
    tp: translate,
    te: exists
  }
}
