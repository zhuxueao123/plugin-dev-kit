<template>
  <main class="brand-login">
    <section class="brand-login__intro">
      <h1>你的项目名称</h1>
      <p>让每一次协作更高效</p>
    </section>
    <form class="brand-login__form" @submit.prevent="loginContext.signIn(form)">
      <h2>欢迎登录</h2>
      <label>账号<input v-model="form.userName" autocomplete="username" required /></label>
      <label>密码<input v-model="form.password" type="password" autocomplete="current-password" required /></label>
      <p v-if="loginContext.state.error" role="alert">{{ loginContext.state.error }}</p>
      <button :disabled="loginContext.state.submitting" type="submit">
        {{ loginContext.state.submitting ? '登录中…' : '登录' }}
      </button>
    </form>
  </main>
</template>

<script setup>
import { reactive } from 'vue'
defineProps({ loginContext: { type: Object, required: true } })
const form = reactive({ userName: '', password: '' })
</script>

<style scoped>
.brand-login { min-height: 100vh; display: flex; align-items: center; justify-content: center; gap: 8vw; padding: 24px; box-sizing: border-box; background: #eff4fb; color: #172b4d; }
.brand-login__intro { max-width: 480px; }
.brand-login__intro h1 { margin: 0 0 16px; font-size: clamp(28px, 4vw, 44px); line-height: 1.25; font-weight: 600; }
.brand-login__intro p { margin: 0; }
.brand-login__form h2 { margin: 0 0 24px; font-size: 24px; line-height: 1.4; font-weight: 600; }
.brand-login__form { width: min(100%, 360px); padding: 32px; box-sizing: border-box; background: white; border-radius: 16px; box-shadow: 0 12px 48px #172b4d12; }
.brand-login__form label { display: block; margin: 20px 0; }
.brand-login__form input { display: block; width: 100%; box-sizing: border-box; padding: 12px; margin-top: 8px; border: 1px solid #ccd5e2; border-radius: 6px; }
.brand-login__form button { width: 100%; padding: 12px; border: 0; border-radius: 6px; background: #245fe5; color: white; cursor: pointer; }
.brand-login__form button:disabled { opacity: .6; cursor: wait; }
.brand-login__form [role=alert] { color: #b42318; }
@media (max-width: 720px) { .brand-login { flex-direction: column; gap: 24px; } }
</style>
