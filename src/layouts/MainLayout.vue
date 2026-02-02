<script setup lang="ts">
// Sidebar removed for Spotlight style
</script>

<template>
    <div class="layout-container">
        <div class="layout-content">
            <slot></slot>
        </div>
    </div>
</template>

<style scoped>
.layout-container {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    min-height: 100vh;
    width: 100vw;
    background-color: transparent;
}

.layout-content {
    width: 100%;
    max-width: 480px;
    position: relative;
    z-index: 1;
}
</style>
