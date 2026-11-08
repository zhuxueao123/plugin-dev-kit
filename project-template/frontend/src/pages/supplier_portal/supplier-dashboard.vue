<template>
  <section class="supplier-dashboard">
    <header class="hero">
      <div>
        <p class="eyebrow">{{ t('pages.portal_eyebrow') }}</p>
        <h1>{{ t('pages.dashboard_title') }}</h1>
        <p class="summary">
          {{ t('pages.dashboard_summary') }}
        </p>
      </div>
    </header>

    <div class="cards">
      <article v-for="card in cards" :key="card.label" class="card">
        <span class="card-label">{{ card.label }}</span>
        <strong class="card-value">{{ card.value }}</strong>
        <span class="card-note">{{ card.note }}</span>
      </article>
    </div>
  </section>
</template>

<script setup>
import { computed } from 'vue'
import { usePluginI18n } from 'src/composables/usePluginI18n'

const { t } = usePluginI18n('supplier_portal')

const cards = computed(() => [
  { label: t('cards.draft_label'), value: '12', note: t('cards.draft_note') },
  { label: t('cards.active_label'), value: '37', note: t('cards.active_note') },
  { label: t('cards.inactive_label'), value: '4', note: t('cards.inactive_note') }
])
</script>

<style scoped>
.supplier-dashboard {
  padding: 32px;
  display: grid;
  gap: 24px;
  background: linear-gradient(180deg, #f7faf8 0%, #eef4f1 100%);
  min-height: 100%;
}

.hero {
  background: #173f35;
  color: #f8fff9;
  padding: 28px 32px;
  border-radius: 20px;
}

.eyebrow {
  margin: 0 0 8px;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  font-size: 12px;
  opacity: 0.7;
}

h1 {
  margin: 0 0 12px;
  font-size: 32px;
}

.summary {
  margin: 0;
  max-width: 720px;
  line-height: 1.6;
}

.cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
}

.card {
  background: #ffffff;
  border-radius: 18px;
  padding: 20px;
  box-shadow: 0 10px 30px rgba(23, 63, 53, 0.08);
  display: grid;
  gap: 10px;
}

.card-label {
  font-size: 13px;
  color: #58766d;
}

.card-value {
  font-size: 36px;
  color: #173f35;
}

.card-note {
  color: #6a7d76;
  line-height: 1.5;
}
</style>
