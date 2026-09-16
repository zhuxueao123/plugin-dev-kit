import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { dirname, resolve, relative } from 'path'
import { fileURLToPath } from 'url'
import fs from 'fs'
import fsPromises from 'fs/promises'

const __dirname = dirname(fileURLToPath(import.meta.url))
const pagesRoot = resolve(__dirname, 'src/pages')
const distRoot = resolve(__dirname, 'dist')
const generatedRoot = resolve(__dirname, '.generated')
const pluginsRoot = resolve(__dirname, '..')

function safeReadJson(filePath) {
  const content = fs.readFileSync(filePath, 'utf8')
  return JSON.parse(content)
}

function loadPluginDescriptors() {
  if (!fs.existsSync(pagesRoot)) {
    return []
  }

  const entries = []
  const dirents = fs.readdirSync(pagesRoot, { withFileTypes: true })
  for (const dirent of dirents) {
    if (!dirent.isDirectory()) {
      continue
    }
    const pluginCode = dirent.name
    const pluginRoot = resolve(pagesRoot, pluginCode)
    const manifestPath = resolve(pluginRoot, 'manifest.json')
    if (!fs.existsSync(manifestPath)) {
      continue
    }

    const manifest = safeReadJson(manifestPath)
    if (!manifest?.pages?.length) {
      continue
    }

    const pages = []
    for (const page of manifest.pages) {
      if (!page?.code || !page?.entry) {
        continue
      }
      const entryPath = resolve(pluginRoot, page.entry)
      if (!fs.existsSync(entryPath)) {
        console.warn(`Plugin ${pluginCode} page ${page.code} entry not found: ${page.entry}`)
        continue
      }
      pages.push({
        code: page.code,
        displayName: page.displayName ?? null,
        route: page.route ?? null,
        entryPath,
        bundleFileName: page.bundle ?? `${page.code}.js`,
        entryExport: page.entryExport ?? 'default',
        propsSchema: page.propsSchema ?? null,
        permissions: Array.isArray(page.permissions) ? page.permissions : [],
        hostFeatures: Array.isArray(page.hostFeatures) ? page.hostFeatures : [],
        layout: page.layout ?? null,
        metadata: page.metadata ?? null
      })
    }

    if (!pages.length) {
      continue
    }

    entries.push({
      pluginCode: manifest.pluginCode ?? pluginCode,
      pluginName: manifest.pluginName ?? null,
      version: manifest.version ?? null,
      pages
    })
  }

  return entries
}

function ensureGeneratedEntry(pluginCode, pageCode, sourceEntryPath) {
  const wrapperDir = resolve(generatedRoot, pluginCode)
  fs.mkdirSync(wrapperDir, { recursive: true })

  const relativeSourcePath = relative(wrapperDir, sourceEntryPath).replace(/\\/g, '/')
  const importPath = relativeSourcePath.startsWith('.') ? relativeSourcePath : `./${relativeSourcePath}`
  const wrapperPath = resolve(wrapperDir, `${pageCode}.js`)
  const wrapperContent = `export { default } from ${JSON.stringify(importPath)}\n`
  fs.writeFileSync(wrapperPath, wrapperContent, 'utf8')
  return wrapperPath
}

function createInputMap(descriptors) {
  const input = {}
  for (const descriptor of descriptors) {
    for (const page of descriptor.pages) {
      const key = `${descriptor.pluginCode}/${page.code}`
      input[key] = ensureGeneratedEntry(descriptor.pluginCode, page.code, page.entryPath)
    }
  }
  return input
}

function resolveBundleFileName(bundle, chunkName) {
  for (const output of Object.values(bundle)) {
    if (output.type === 'chunk' && output.name === chunkName) {
      return output.fileName
    }
  }
  return null
}

function manifestWriterPlugin(descriptors) {
  return {
    name: 'asapflow-plugin-manifest-writer',
    async generateBundle(_options, bundle) {
      if (!descriptors.length) {
        return
      }

      const manifestsDir = resolve(distRoot, 'manifests')
      await fsPromises.mkdir(manifestsDir, { recursive: true })

      const summary = []

      for (const descriptor of descriptors) {
        const pages = []
        for (const page of descriptor.pages) {
          const chunkName = `${descriptor.pluginCode}/${page.code}`
          const fileName = resolveBundleFileName(bundle, chunkName)
          if (!fileName) {
            this.warn?.(`未找到插件页面 ${chunkName} 的构建产物`)
            continue
          }
          const normalizedFileName = fileName.replace(/\\/g, '/')
          const bundlePath = `/plugins/frontend/dist/${normalizedFileName}`
          const modulePath = relative(pluginsRoot, page.entryPath).replace(/\\/g, '/')
          pages.push({
            code: page.code,
            displayName: page.displayName,
            route: page.route,
            modulePath,
            bundlePath,
            entryExport: page.entryExport,
            propsSchema: page.propsSchema,
            permissions: page.permissions,
            hostFeatures: page.hostFeatures,
            layout: page.layout,
            metadata: page.metadata
          })
        }

        const pluginManifest = {
          pluginCode: descriptor.pluginCode,
          pluginName: descriptor.pluginName,
          version: descriptor.version,
          pages
        }

        const manifestPath = resolve(manifestsDir, `${descriptor.pluginCode}.json`)
        await fsPromises.writeFile(manifestPath, JSON.stringify(pluginManifest, null, 2), 'utf8')
        summary.push({
          pluginCode: descriptor.pluginCode,
          pluginName: descriptor.pluginName,
          version: descriptor.version,
          manifest: relative(distRoot, manifestPath).replace(/\\/g, '/'),
          pages: pluginManifest.pages.length
        })
      }

      const indexPath = resolve(manifestsDir, 'index.json')
      await fsPromises.writeFile(indexPath, JSON.stringify({
        generatedAt: new Date().toISOString(),
        plugins: summary
      }, null, 2), 'utf8')
    }
  }
}

function cssInjectorPlugin() {
  return {
    name: 'asapflow-plugin-css-injector',
    enforce: 'post',
    generateBundle(_options, bundle) {
      for (const output of Object.values(bundle)) {
        if (output.type !== 'chunk' || !output.isEntry) {
          continue
        }
        const cssFiles = [...(output.viteMetadata?.importedCss || [])]
        if (!cssFiles.length) {
          continue
        }
        const imports = cssFiles.map((cssFile) => {
          const cssRelativePath = relative(dirname(output.fileName), cssFile).replace(/\\/g, '/')
          const normalizedPath = cssRelativePath.startsWith('.') ? cssRelativePath : `./${cssRelativePath}`
          return `for(const href of [new URL(${JSON.stringify(normalizedPath)},import.meta.url).href]){if(!document.querySelector('link[href="'+href+'"]')){const link=document.createElement('link');link.rel='stylesheet';link.href=href;document.head.appendChild(link)}}\n`
        }).join('')
        output.code = imports + output.code
      }
    }
  }
}

const descriptors = loadPluginDescriptors()
const rollupInput = descriptors.length ? createInputMap(descriptors) : undefined

export default defineConfig({
  plugins: [vue(), cssInjectorPlugin(), manifestWriterPlugin(descriptors)],
  resolve: {
    alias: {
      src: resolve(__dirname, 'src')
    }
  },
  build: {
    outDir: distRoot,
    emptyOutDir: true,
    target: 'es2020',
    cssCodeSplit: true,
    rollupOptions: {
      input: rollupInput,
      preserveEntrySignatures: 'strict',
      output: {
        format: 'es',
        entryFileNames: ({ name }) => `${name}-[hash].js`,
        chunkFileNames: 'shared/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash][extname]'
      }
    }
  }
})
