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
    align-items: flex-start; /* Start from top to allow expansion */
    min-height: 100vh;
    width: 100vw;
    background-color: transparent; /* Allow body bg to show or use tauri transparent */
    padding-top: 2vh; /* Search bar closer to top */
}

.layout-content {
    width: 100%;
    max-width: 750px;
    position: relative;
    z-index: 1;
}
</style>
