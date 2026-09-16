<template>
  <section class="status-board">
    <header>
      <p class="eyebrow">{{ t('pages.status_board_eyebrow') }}</p>
      <h1>{{ t('pages.status_board_title') }}</h1>
    </header>

    <ul class="lanes">
      <li v-for="lane in lanes" :key="lane.title" class="lane">
        <div class="lane-head">
          <span>{{ lane.title }}</span>
          <strong>{{ lane.count }}</strong>
        </div>
        <p>{{ lane.description }}</p>
      </li>
    </ul>
  </section>
</template>

<script setup>
import { computed } from 'vue'
import { usePluginI18n } from 'src/composables/usePluginI18n'

const { t } = usePluginI18n('supplier_portal')

const lanes = computed(() => [
  { title: t('lanes.draft_title'), count: 12, description: t('lanes.draft_description') },
  { title: t('lanes.active_title'), count: 37, description: t('lanes.active_description') },
  { title: t('lanes.inactive_title'), count: 4, description: t('lanes.inactive_description') }
])
</script>

<style scoped>
.status-board {
  padding: 28px;
  display: grid;
  gap: 18px;
  min-height: 100%;
  background: #f5f1ea;
}

.eyebrow {
  margin: 0 0 6px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 12px;
  color: #7f6a4f;
}

h1 {
  margin: 0;
  font-size: 30px;
  color: #3e3020;
}

.lanes {
  list-style: none;
  padding: 0;
  margin: 0;
  display: grid;
  gap: 14px;
}

.lane {
  padding: 18px 20px;
  border-radius: 16px;
  background: #ffffff;
  box-shadow: 0 8px 24px rgba(62, 48, 32, 0.08);
}

.lane-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-weight: 600;
  color: #3e3020;
}

p {
  margin: 0;
  line-height: 1.6;
  color: #6a5b4a;
}
</style>
