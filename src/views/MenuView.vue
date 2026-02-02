<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const searchQuery = ref('');
const selectedIndex = ref(0);
const searchInput = ref<HTMLInputElement | null>(null);
const isSearching = ref(false);

// Search results from backend
const fileResults = ref<any[]>([]);
const appResults = ref<any[]>([]);

const groupedResults = computed(() => {
	const groups = [];
	let currentIndex = 0;

	// 1. Applications Group (First)
	if (appResults.value.length > 0) {
		groups.push({
			id: 'apps',
			label: 'Applications',
			items: appResults.value,
			startIndex: currentIndex
		});
		currentIndex += appResults.value.length;
	}

	// 2. Web Search Group (Second)
	if (searchQuery.value) {
		groups.push({
			id: 'web',
			label: 'Web Search',
			items: [{
				type: 'web',
				name: `Search Google for "${searchQuery.value}"`,
				description: 'Open in browser',
				icon: 'pi pi-google',
				action: () => {
					invoke('open_item', { path: `https://www.google.com/search?q=${encodeURIComponent(searchQuery.value)}` });
					setTimeout(() => invoke('exit_app'), 100);
				}
			}],
			startIndex: currentIndex
		});
		currentIndex += 1;
	}

	// 3. Files Group (Third)
	if (fileResults.value.length > 0) {
		groups.push({
			id: 'files',
			label: 'Files & Directories',
			items: fileResults.value,
			startIndex: currentIndex
		});
	}

	return groups;
});

let searchTimeout: any = null;

const onSearchInput = () => {
	selectedIndex.value = 0;

	if (searchTimeout) {
		clearTimeout(searchTimeout);
	}

	searchTimeout = setTimeout(() => {
		performSearch();
	}, 150);
};

const performSearch = async () => {
	if (!searchQuery.value) {
		fileResults.value = [];
		appResults.value = [];
		return;
	}

	isSearching.value = true;

	try {
		const promises = [];

		// Always search Apps
		promises.push(
			invoke<any[]>('search_apps_command', {
				query: searchQuery.value,
				maxResults: 20
			}).then(results => {
				appResults.value = results.map(r => ({
					...r,
					description: r.path,
				}));
			}).catch(err => {
				appResults.value = [];
			})
		);

		// Always search Files
		promises.push(
			invoke<any[]>('search_files_command', {
				query: searchQuery.value,
				maxResults: 10
			}).then(results => {
				fileResults.value = results.map(r => ({
					...r,
					description: r.path,
				}));
			}).catch(err => {
				fileResults.value = [];
			})
		);

		await Promise.all(promises);
	} finally {
		isSearching.value = false;
	}
};

const executeAction = async (item: any) => {
	if (item.type === 'web' || (item.type === 'action' && item.action)) {
		if (item.action) item.action();
		return;
	}

	if (item.path) {
		try {
			await invoke('open_item', { path: item.path });
			setTimeout(() => invoke('exit_app'), 100);
		} catch (err) { }
	}
};

const exitApp = () => {
	invoke('exit_app');
}

const getIconStyle = (item: any) => {
	if (item.type === 'web') return 'color: #4285F4';
	if (!item.name) return '';
	const name = item.name.toLowerCase();
	if (name.includes('firefox')) return 'color: #ff9500';
	if (name.includes('code')) return 'color: #007acc';
	if (name.includes('spotify')) return 'color: #1bd860';
	if (name.includes('steam')) return 'color: #171a21';
	if (name.includes('discord')) return 'color: #5865F2';
	return '';
};

// Scroll to selected
const scrollToSelected = () => {
	nextTick(() => {
		const selectedElement = document.querySelector('.app-item.selected');
		if (selectedElement) {
			selectedElement.scrollIntoView({
				behavior: 'smooth',
				block: 'nearest',
				inline: 'nearest'
			});
		}
	});
};

const onKeydown = (e: KeyboardEvent) => {
	// Navigate using flattened groups source of truth
	const flatList = groupedResults.value.flatMap(g => g.items);
	const maxIndex = flatList.length - 1;

	if (e.key === 'ArrowDown') {
		e.preventDefault();
		if (flatList.length === 0) return;
		selectedIndex.value = Math.min(selectedIndex.value + 1, maxIndex);
		scrollToSelected();
	} else if (e.key === 'ArrowUp') {
		e.preventDefault();
		if (flatList.length === 0) return;
		selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
		scrollToSelected();
	} else if (e.key === 'Enter') {
		e.preventDefault();
		if (flatList[selectedIndex.value]) {
			executeAction(flatList[selectedIndex.value]);
		}
	} else if (e.key === 'Escape') {
		invoke('exit_app');
	}
};

onMounted(() => {
	window.addEventListener('keydown', onKeydown);
	if (searchInput.value) searchInput.value.focus();
});

onUnmounted(() => {
	window.removeEventListener('keydown', onKeydown);
	if (searchTimeout) clearTimeout(searchTimeout);
});
</script>

<template>
	<div class="menu-container glass-panel">
		<!-- Header -->
		<header class="menu-header">
			<div class="header-brand">
				<i class="pi pi-compass brand-icon" />
				<span class="brand-text">Applications</span>
			</div>
			<button class="close-btn" @click="exitApp">
				<i class="pi pi-times" style="font-size: 0.7rem;"></i>
				<span>Close</span>
			</button>
		</header>

		<!-- Search Area -->
		<div class="search-container">
			<input ref="searchInput" v-model="searchQuery" type="text" class="search-input"
				placeholder="Type to search..." @input="onSearchInput" />
		</div>

		<!-- Content List -->
		<div class="list-container custom-scroll">
			<div v-if="isSearching" class="status-msg">
				<i class="pi pi-spinner pi-spin"></i>
			</div>

			<!-- Grouped View -->
			<div v-else-if="groupedResults.length > 0" class="results-wrapper">
				<div v-for="group in groupedResults" :key="group.id" class="result-group">
					<div class="group-title">{{ group.label }}</div>
					<ul class="app-list">
						<li v-for="(item, index) in group.items" :key="index" class="app-item"
							:class="{ selected: (group.startIndex + index) === selectedIndex }"
							@mouseenter="selectedIndex = (group.startIndex + index)" @click="executeAction(item)">
							<div class="app-icon-wrapper">
								<i :class="item.icon" class="app-icon-main" :style="getIconStyle(item)"></i>
							</div>
							<div class="app-info">
								<div class="app-title">{{ item.name }}</div>
								<div class="app-desc">{{ item.description }}</div>
							</div>
						</li>
					</ul>
				</div>
			</div>

			<div v-else class="status-msg empty-state">
				<template v-if="searchQuery">
					<span>No results found</span>
				</template>
				<template v-else>
					<span>Start typing to search...</span>
				</template>
			</div>
		</div>
	</div>
</template>

<style scoped>
/* Main Container */
.menu-container {
	display: flex;
	flex-direction: column;
	width: 100%;
	height: 100vh;
	overflow: hidden;
	font-family: 'Inter', sans-serif;
	color: white;
}

/* Glass Effect */
.glass-panel {
	background: linear-gradient(135deg, rgba(20, 20, 30, 0.85), rgba(30, 30, 40, 0.95));
	backdrop-filter: blur(40px);
	-webkit-backdrop-filter: blur(40px);
	border: 1px solid rgba(255, 255, 255, 0.08);
	box-shadow: 0 30px 60px rgba(0, 0, 0, 0.5);
}

/* Header */
.menu-header {
	display: flex;
	justify-content: space-between;
	align-items: center;
	padding: 20px 24px 10px 24px;
}

.header-brand {
	display: flex;
	align-items: center;
	gap: 10px;
}

.brand-icon {
	font-size: 1.2rem;
	color: rgba(255, 255, 255, 0.9);
}

.brand-text {
	font-size: 1.1rem;
	font-weight: 600;
	letter-spacing: 0.5px;
	color: rgba(255, 255, 255, 0.95);
}

.close-btn {
	display: flex;
	align-items: center;
	gap: 6px;
	background: rgba(255, 255, 255, 0.05);
	border: 1px solid rgba(255, 255, 255, 0.1);
	padding: 6px 14px;
	border-radius: 20px;
	color: rgba(255, 255, 255, 0.6);
	font-size: 0.85rem;
	cursor: pointer;
	transition: all 0.2s ease;
}

.close-btn:hover {
	background: rgba(255, 255, 255, 0.15);
	color: white;
}

/* Search */
.search-container {
	padding: 10px 24px;
}

.search-input {
	width: 100%;
	background: rgba(0, 0, 0, 0.2);
	border: none;
	padding: 12px 16px;
	border-radius: 12px;
	color: white;
	font-size: 0.95rem;
	outline: none;
	transition: background 0.2s;
}

.search-input:focus {
	background: rgba(0, 0, 0, 0.3);
}

.search-input::placeholder {
	color: rgba(255, 255, 255, 0.2);
}

/* List */
.list-container {
	flex: 1;
	overflow-y: auto;
	padding: 10px 0;
}

.app-list {
	list-style: none;
	padding: 0;
	margin: 0;
}

.app-item {
	display: flex;
	align-items: center;
	padding: 12px 24px;
	cursor: pointer;
	border-bottom: 1px solid rgba(255, 255, 255, 0.03);
	transition: background 0.2s;
}

.app-item:last-child {
	border-bottom: none;
}

.app-item:hover,
.app-item.selected {
	background: rgba(255, 255, 255, 0.06);
}

.app-icon-wrapper {
	width: 42px;
	height: 42px;
	display: flex;
	align-items: center;
	justify-content: center;
	background: rgba(255, 255, 255, 0.05);
	border-radius: 10px;
	margin-right: 16px;
	font-size: 1.5rem;
}

.app-icon-main {
	color: rgba(255, 255, 255, 0.9);
}

.app-info {
	display: flex;
	flex-direction: column;
	gap: 4px;
	overflow: hidden;
}

.app-title {
	font-size: 0.95rem;
	font-weight: 500;
	color: rgba(255, 255, 255, 0.95);
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.app-desc {
	font-size: 0.8rem;
	color: rgba(255, 255, 255, 0.5);
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

/* Group Headers */
.group-title {
	padding: 12px 24px 6px 24px;
	font-size: 0.7rem;
	font-weight: 600;
	text-transform: uppercase;
	letter-spacing: 0.8px;
	color: rgba(255, 255, 255, 0.4);
	margin-top: 4px;
}

.result-group:first-child .group-title {
	margin-top: 0;
}

.result-group {
	margin-bottom: 4px;
}

/* Status States */
.status-msg {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	height: 100px;
	color: rgba(255, 255, 255, 0.3);
	font-size: 0.9rem;
}

.empty-state {
	height: 150px;
}

/* Scrollbar */
.custom-scroll::-webkit-scrollbar {
	width: 4px;
}

.custom-scroll::-webkit-scrollbar-track {
	background: transparent;
}

.custom-scroll::-webkit-scrollbar-thumb {
	background: rgba(255, 255, 255, 0.1);
	border-radius: 4px;
}

.custom-scroll::-webkit-scrollbar-thumb:hover {
	background: rgba(255, 255, 255, 0.2);
}
</style>
