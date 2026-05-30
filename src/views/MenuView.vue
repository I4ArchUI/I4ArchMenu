<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const searchQuery = ref('');
const selectedIndex = ref(0);
const searchInput = ref<HTMLInputElement | null>(null);
const isSearching = ref(false);

// Theme management
const updateTheme = async () => {
	try {
		const theme = await invoke<string>('get_gtk_theme');
		// gsettings returns 'prefer-dark', 'default', or 'prefer-light'
		if (theme.includes('dark')) {
			document.documentElement.setAttribute('data-theme', 'dark');
		} else {
			document.documentElement.removeAttribute('data-theme');
		}
	} catch (e) { }
};

let themeInterval: any = null;

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
				item_type: 'web',
				name: `Search Google for "${searchQuery.value}"`,
				description: 'Open search query in default web browser',
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

		// Always search Apps (both System and Flatpak)
		promises.push(
			invoke<any[]>('search_apps_command', {
				query: searchQuery.value,
				maxResults: 20
			}).then(results => {
				appResults.value = results.map(r => {
					const filename = r.path.split('/').pop()?.replace('.desktop', '') || r.path;
					return {
						...r,
						description: r.item_type === 'flatpak' 
							? `Flatpak App • ${filename}` 
							: `System App • ${filename}`,
					};
				});
			}).catch(_err => {
				appResults.value = [];
			})
		);

		// Always search Files
		promises.push(
			invoke<any[]>('search_files_command', {
				query: searchQuery.value,
				maxResults: 10
			}).then(results => {
				fileResults.value = results.map(r => {
					const label = r.item_type === 'folder' ? 'Folder' : 'File';
					return {
						...r,
						description: `${label} • ${r.path}`,
					};
				});
			}).catch(_err => {
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

const getBadgeLabel = (type: string) => {
	if (type === 'flatpak') return 'Flatpak';
	if (type === 'app') return 'System';
	if (type === 'file') return 'File';
	if (type === 'folder') return 'Folder';
	if (type === 'web') return 'Web';
	return type;
};

const getIconStyle = (_item: any) => {
	return 'color: #e5c197'; // Only use premium yellow/amber
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
	updateTheme();
	themeInterval = setInterval(updateTheme, 1000);
});

onUnmounted(() => {
	window.removeEventListener('keydown', onKeydown);
	if (searchTimeout) clearTimeout(searchTimeout);
	if (themeInterval) clearInterval(themeInterval);
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
				<i class="pi pi-times" style="font-size: 0.75rem;"></i>
				<span>Close</span>
			</button>
		</header>

		<!-- Search Area -->
		<div class="search-container">
			<div class="search-wrapper">
				<i class="pi pi-search search-bar-icon"></i>
				<input ref="searchInput" v-model="searchQuery" type="text" class="search-input"
					placeholder="Type to search apps, files or web..." @input="onSearchInput" />
			</div>
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
								<div class="app-title-row">
									<span class="app-title">{{ item.name }}</span>
									<span :class="['item-badge', item.item_type]">{{ getBadgeLabel(item.item_type) }}</span>
								</div>
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
	font-family: var(--font-main);
	color: var(--text-main);
}

/* Glass Effect */
.glass-panel {
	background: rgba(245, 240, 230, 0.88); /* More solid light glass, less transparent */
	backdrop-filter: blur(20px) saturate(140%);
	-webkit-backdrop-filter: blur(20px) saturate(140%);
	border: 1px solid rgba(229, 193, 151, 0.35); /* Slightly more defined yellow border */
	box-shadow: 
		0 20px 40px rgba(0, 0, 0, 0.08), 
		inset 0 1px 0 rgba(255, 255, 255, 0.5); /* Highlight at top */
	border-radius: 20px;
}

:root[data-theme="dark"] .glass-panel {
	background: rgba(22, 20, 16, 0.88); /* More solid dark smoked glass, less transparent */
	border: 1px solid rgba(229, 193, 151, 0.22);
	box-shadow: 
		0 30px 60px rgba(0, 0, 0, 0.4), 
		inset 0 1px 0 rgba(255, 255, 255, 0.08); /* Highlight at top */
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
	font-size: 1.3rem;
	color: #e5c197; /* Premium Amber/Yellow */
	filter: drop-shadow(0 0 8px rgba(229, 193, 151, 0.35));
}

.brand-text {
	font-size: 1.15rem;
	font-weight: 700;
	letter-spacing: 0.5px;
	color: #e5c197; /* Solid yellow, no gradient */
}

.close-btn {
	display: flex;
	align-items: center;
	gap: 6px;
	background: rgba(229, 193, 151, 0.08); /* Translucent yellow background */
	border: 1px solid rgba(229, 193, 151, 0.2);
	padding: 6px 14px;
	border-radius: 20px;
	color: #e5c197; /* Yellow */
	font-size: 0.85rem;
	cursor: pointer;
	transition: all 0.2s ease;
}

.close-btn:hover {
	background: rgba(229, 193, 151, 0.16);
	color: #e5c197;
	box-shadow: 0 0 10px rgba(229, 193, 151, 0.15);
}

/* Search */
.search-container {
	padding: 10px 24px;
}

.search-wrapper {
	position: relative;
	width: 100%;
}

.search-bar-icon {
	position: absolute;
	left: 16px;
	top: 50%;
	transform: translateY(-50%);
	color: var(--text-muted);
	font-size: 1rem;
	pointer-events: none;
}

.search-input {
	width: 100%;
	background: rgba(255, 255, 255, 0.03); /* Glass input */
	border: 1px solid rgba(229, 193, 151, 0.12);
	padding: 14px 18px 14px 44px; /* Space for search icon */
	border-radius: 14px;
	color: var(--text-main);
	font-size: 0.95rem;
	outline: none;
	transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
	box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.05);
}

:root[data-theme="dark"] .search-input {
	background: rgba(0, 0, 0, 0.15);
}

.search-input:focus {
	background: rgba(255, 255, 255, 0.06);
	border-color: #e5c197; /* Yellow focus border */
	box-shadow: 0 0 16px rgba(229, 193, 151, 0.15), inset 0 1px 2px rgba(0, 0, 0, 0.05);
}

:root[data-theme="dark"] .search-input:focus {
	background: rgba(0, 0, 0, 0.22);
}

.search-input::placeholder {
	color: var(--search-placeholder);
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
	border-bottom: 1px solid var(--border-color);
	transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
	position: relative;
	overflow: hidden;
}

.app-item::before {
	content: '';
	position: absolute;
	left: 0;
	top: 0;
	bottom: 0;
	width: 3px;
	background: #e5c197; /* Yellow active indicator */
	transform: scaleY(0);
	transition: transform 0.25s ease;
}

.app-item.selected::before,
.app-item:hover::before {
	transform: scaleY(1);
}

.app-item:last-child {
	border-bottom: none;
}

.app-item.selected,
.app-item:hover {
	background: rgba(229, 193, 151, 0.06); /* Soft glass reflection with yellow tint */
	transform: translateX(4px);
}

.app-icon-wrapper {
	width: 42px;
	height: 42px;
	display: flex;
	align-items: center;
	justify-content: center;
	background: rgba(229, 193, 151, 0.05);
	border: 1px solid rgba(229, 193, 151, 0.08);
	border-radius: 12px;
	margin-right: 16px;
	font-size: 1.4rem;
	transition: all 0.25s ease;
}

.app-item.selected .app-icon-wrapper,
.app-item:hover .app-icon-wrapper {
	transform: scale(1.05);
	background: rgba(229, 193, 151, 0.1);
	border-color: rgba(229, 193, 151, 0.2);
}

.app-icon-main {
	color: var(--icon-color);
}

.app-info {
	display: flex;
	flex-direction: column;
	gap: 4px;
	overflow: hidden;
	flex: 1;
}

.app-title-row {
	display: flex;
	align-items: center;
	justify-content: space-between;
	padding-right: 8px;
}

.app-title {
	font-size: 0.95rem;
	font-weight: 600;
	color: var(--text-main);
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.app-desc {
	font-size: 0.8rem;
	color: var(--text-secondary);
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
	opacity: 0.85;
}

/* Unified Premium Yellow Badges */
.item-badge {
	font-size: 9px;
	font-weight: 700;
	padding: 2px 7px;
	border-radius: 8px;
	text-transform: uppercase;
	letter-spacing: 0.5px;
	background: rgba(229, 193, 151, 0.12);
	color: #e5c197;
	border: 1px solid rgba(229, 193, 151, 0.22);
}

/* Group Headers */
.group-title {
	padding: 12px 24px 6px 24px;
	font-size: 0.7rem;
	font-weight: 700;
	text-transform: uppercase;
	letter-spacing: 0.8px;
	color: var(--text-muted);
	margin-top: 6px;
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
	color: var(--text-muted);
	font-size: 0.95rem;
}

.empty-state {
	height: 150px;
}

/* Scrollbar */
.custom-scroll::-webkit-scrollbar {
	width: 4px;
}

.custom-scroll::-webkit-scrollbar-track {
	background: var(--scrollbar-track);
}

.custom-scroll::-webkit-scrollbar-thumb {
	background: var(--scrollbar-thumb);
	border-radius: 4px;
}

.custom-scroll::-webkit-scrollbar-thumb:hover {
	background: var(--scrollbar-thumb);
	opacity: 0.8;
}
</style>

