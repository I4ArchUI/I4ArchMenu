<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';

const router = useRouter();
const searchQuery = ref('');
const selectedIndex = ref(0);
const searchInput = ref<HTMLInputElement | null>(null);
const currentCategory = ref('all'); // 'all', 'apps', 'files'
const isSearching = ref(false);

// Search results from backend
const fileResults = ref<any[]>([]);
const appResults = ref<any[]>([]);

interface Category {
  id: string;
  label: string;
  icon: string;
}

const categories: Category[] = [
  { id: 'all', label: 'All', icon: 'pi pi-th-large' },
  { id: 'apps', label: 'Apps', icon: 'pi pi-desktop' },
  { id: 'files', label: 'Files', icon: 'pi pi-folder-open' },
];

const allResults = computed(() => {
  if (currentCategory.value === 'apps') {
    return appResults.value;
  } else if (currentCategory.value === 'files') {
    return fileResults.value;
  } else {
    return [...appResults.value, ...fileResults.value];
  }
});

// Grouped results for "All" category view
const groupedResults = computed(() => {
  if (currentCategory.value !== 'all') {
    return [];
  }
  
  const groups = [];
  
  if (appResults.value.length > 0) {
    groups.push({
      title: 'Applications',
      icon: 'pi pi-desktop',
      items: appResults.value,
      startIndex: 0
    });
  }
  
  if (fileResults.value.length > 0) {
    groups.push({
      title: 'Files & Folders',
      icon: 'pi pi-folder-open',
      items: fileResults.value,
      startIndex: appResults.value.length
    });
  }
  
  return groups;
});

let searchTimeout: any = null;

const onSearchInput = () => {
  selectedIndex.value = 0;
  
  // Debounce search
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  
  searchTimeout = setTimeout(() => {
    performSearch();
  }, 150); // Optimized for fast response
};

const performSearch = async () => {
  if (!searchQuery.value || searchQuery.value.length < 1) {
    fileResults.value = [];
    appResults.value = [];
    return;
  }

  isSearching.value = true;

  try {
    const promises = [];
    
    if (currentCategory.value === 'all' || currentCategory.value === 'files') {
      promises.push(
        invoke<any[]>('search_files_command', { 
          query: searchQuery.value,
          maxResults: 20 
        }).then(results => {
          fileResults.value = results.map(r => ({
            ...r,
            description: r.path,
          }));
        }).catch(err => {
          console.error('File search error:', err);
          fileResults.value = [];
        })
      );
    }

    if (currentCategory.value === 'all' || currentCategory.value === 'apps') {
      promises.push(
        invoke<any[]>('search_apps_command', { 
          query: searchQuery.value,
          maxResults: 20
        }).then(results => {
          appResults.value = results.map(r => ({
            ...r,
            description: 'Application',
          }));
        }).catch(err => {
          console.error('App search error:', err);
          appResults.value = [];
        })
      );
    }

    await Promise.all(promises);
  } finally {
    isSearching.value = false;
  }
};

const executeAction = async (item: any) => {
  if (item.type === 'route' && item.route) {
    router.push(item.route);
  } else if (item.type === 'action' && item.action) {
    item.action();
  } else if (item.path) {
    try {
      await invoke('open_item', { path: item.path });
      setTimeout(() => {
        invoke('exit_app');
      }, 100);
    } catch (err) {
      console.error('Failed to open item:', err);
    }
  }
};

const clearSearch = () => {
  searchQuery.value = '';
  fileResults.value = [];
  appResults.value = [];
  selectedIndex.value = 0;
};

const getIconClass = (item: any) => {
  if (item.item_type === 'app') return 'icon-app';
  if (item.item_type === 'folder') return 'icon-folder';
  return 'icon-file';
};

const scrollToSelected = () => {
  nextTick(() => {
    const selectedElement = document.querySelector('.result-item.selected');
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
  const maxIndex = allResults.value.length - 1;
  
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, maxIndex);
    scrollToSelected();
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
    scrollToSelected();
  } else if (e.key === 'Enter') {
    e.preventDefault();
    if (allResults.value[selectedIndex.value]) {
      executeAction(allResults.value[selectedIndex.value]);
    }
  } else if (e.key === 'Escape') {
    if (searchQuery.value) {
      clearSearch();
      e.stopPropagation();
    } else {
      invoke('exit_app');
    }
  }
};

onMounted(() => {
  window.addEventListener('keydown', onKeydown);
  if(searchInput.value) searchInput.value.focus();
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown);
  if (searchTimeout) clearTimeout(searchTimeout);
});
</script>

<template>
  <div class="spotlight-container">
    <div class="search-bar glass-effect">
      <div class="search-input-wrapper">
        <i class="pi pi-search search-icon" />
        <input 
          ref="searchInput"
          v-model="searchQuery" 
          type="text" 
          :placeholder="currentCategory === 'all' ? 'Search apps, files...' : currentCategory === 'apps' ? 'Search applications...' : 'Search files...'"
          class="spotlight-input"
          autofocus
          @input="onSearchInput"
        />
        <div v-if="searchQuery" class="clear-icon" @click="clearSearch">
            <i class="pi pi-times" />
        </div>
      </div>
      
      <div class="category-tabs">
        <button 
          v-for="cat in categories" 
          :key="cat.id"
          class="category-tab"
          :class="{ active: currentCategory === cat.id }"
          @click="currentCategory = cat.id; performSearch()"
        >
          <i :class="cat.icon" />
          <span>{{ cat.label }}</span>
        </button>
      </div>
    </div>

    <!-- Loading Indicator -->
    <div v-if="isSearching" class="loading-container glass-effect">
      <i class="pi pi-spin pi-spinner" style="font-size: 2rem; color: rgba(255,255,255,0.6)"></i>
      <span class="loading-text">Searching...</span>
    </div>

    <!-- Results Container -->
    <div v-else-if="allResults.length > 0" class="results-container glass-effect">
      <!-- Grouped view for "All" category -->
      <template v-if="currentCategory === 'all' && groupedResults.length > 0">
        <div v-for="group in groupedResults" :key="group.title" class="result-group">
          <div class="group-header">
            <i :class="group.icon" style="margin-right: 8px"></i>
            <span>{{ group.title }}</span>
            <span class="group-count">{{ group.items.length }}</span>
          </div>
          <ul class="results-list">
            <li 
              v-for="(item, itemIndex) in group.items" 
              :key="item.path + itemIndex"
              class="result-item"
              :class="{ 'selected': (group.startIndex + itemIndex) === selectedIndex }"
              @click="executeAction(item)"
              @mouseenter="selectedIndex = group.startIndex + itemIndex"
            >
              <div class="item-icon" :class="getIconClass(item)">
                <i :class="item.icon" style="font-size: 0.9rem"></i>
              </div>
              <div class="item-content">
                <div class="item-title">{{ item.name || item.label }}</div>
                <div class="item-desc">{{ item.description || item.path }}</div>
              </div>
              <div class="item-shortcut">
                 <span class="shortcut-key">↵</span>
              </div>
            </li>
          </ul>
        </div>
      </template>
      
      <!-- Flat view for specific categories -->
      <ul v-else class="results-list">
        <li 
          v-for="(item, index) in allResults" 
          :key="item.path + index"
          class="result-item"
          :class="{ 'selected': index === selectedIndex }"
          @click="executeAction(item)"
          @mouseenter="selectedIndex = index"
        >
          <div class="item-icon" :class="getIconClass(item)">
            <i :class="item.icon" style="font-size: 0.9rem"></i>
          </div>
          <div class="item-content">
            <div class="item-title">{{ item.name || item.label }}</div>
            <div class="item-desc">{{ item.description || item.path }}</div>
          </div>
          <div class="item-shortcut">
             <span class="shortcut-key">↵</span>
          </div>
        </li>
      </ul>
    </div>
    
    <!-- No Results -->
    <div v-else-if="searchQuery && !isSearching" class="no-results glass-effect">
        <div class="flex flex-col items-center justify-center p-6 text-surface-500">
            <i class="pi pi-filter-slash mb-3" style="font-size: 3rem; opacity: 0.4"></i>
            <span class="text-lg">No results found</span>
            <span class="text-sm opacity-60 mt-2">Try a different search term</span>
        </div>
    </div>


    <!-- Empty state hint -->
    <div v-else-if="!searchQuery" class="empty-hint glass-effect">
      <div class="hint-content">
        <i class="pi pi-search" style="font-size: 2rem; opacity: 0.3; margin-bottom: 12px"></i>
        <span class="hint-text">Start typing to search apps and files...</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.spotlight-container {
  padding: 10px;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}


.search-bar {
  border-radius: 4px;
  padding: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.search-input-wrapper {
  display: flex;
  align-items: center;
  padding: 0 14px;
  height: 46px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.search-icon {
  font-size: 1.1rem;
  color: rgba(255, 255, 255, 0.5);
  margin-right: 12px;
}

.spotlight-input {
  flex: 1;
  background: transparent;
  border: none;
  font-size: 1.1rem;
  font-weight: 400;
  color: white;
  outline: none;
  height: 100%;
}

.spotlight-input::placeholder {
  color: rgba(255, 255, 255, 0.25);
}

.clear-icon {
  cursor: pointer;
  color: rgba(255, 255, 255, 0.4);
  transition: all 0.2s;
  padding: 8px;
  border-radius: 50%;
}

.clear-icon:hover {
  color: white;
  background: rgba(255, 255, 255, 0.1);
}

/* Category Tabs */
.category-tabs {
  display: flex;
  padding: 6px;
  gap: 6px;
  background: rgba(0, 0, 0, 0.2);
}

.category-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 6px 10px;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.category-tab:hover {
  background: rgba(255, 255, 255, 0.05);
  color: rgba(255, 255, 255, 0.8);
}

.category-tab.active {
  background: rgba(0, 122, 255, 0.6);
  border-color: rgba(0, 122, 255, 0.8);
  color: white;
}

/* Loading */
.loading-container {
  border-radius: 16px;
  padding: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.loading-text {
  color: rgba(255, 255, 255, 0.6);
  font-size: 1rem;
}

/* Results */
.results-container {
  border-radius: 4px;
  height: calc(100vh - 150px);
  overflow-y: auto;
  padding: 4px;
}

.result-group {
  margin-bottom: 12px;
}

.result-group:last-child {
  margin-bottom: 0;
}

.group-header {
  display: flex;
  align-items: center;
  padding: 6px 10px;
  font-size: 0.7rem;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  margin-bottom: 2px;
}

.group-count {
  margin-left: auto;
  background: rgba(255, 255, 255, 0.1);
  padding: 1px 6px;
  border-radius: 10px;
  font-size: 0.65rem;
}

.no-results {
  border-radius: 4px;
  padding: 40px;
}

.empty-hint {
  border-radius: 4px;
  padding: 50px 20px;
}

.hint-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.hint-text {
  color: rgba(255, 255, 255, 0.4);
  font-size: 0.9rem;
}

.results-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.result-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.15s ease;
  color: rgba(255, 255, 255, 0.85);
  margin-bottom: 1px;
}

.result-item:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

.result-item.selected {
  background: rgba(0, 122, 255, 0.8);
  color: white;
}

.item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  margin-right: 8px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.08);
  transition: all 0.2s;
}

.result-item.selected .item-icon {
  background: rgba(255, 255, 255, 0.2);
}

.icon-app {
  background: rgba(0, 122, 255, 0.2);
}

.icon-folder {
  background: rgba(255, 193, 7, 0.2);
}

.icon-file {
  background: rgba(76, 175, 80, 0.2);
}

.item-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: 8px;
}

.item-title {
  font-size: 0.8rem;
  line-height: 1;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-desc {
  font-size: 0.68rem;
  opacity: 0.6;
  line-height: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-type-badge {
  display: inline-block;
  margin-top: 4px;
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  width: fit-content;
}

.item-shortcut {
  opacity: 0.4;
  font-size: 0.85rem;
  margin-left: 12px;
}

.shortcut-key {
  display: inline-block;
  padding: 2px 6px;
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: 2px;
  font-weight: 600;
  font-size: 0.7rem;
  background: rgba(255, 255, 255, 0.05);
}

.result-item.selected .shortcut-key {
  border-color: rgba(255, 255, 255, 0.6);
  background: rgba(255, 255, 255, 0.15);
}

/* Scrollbar */
.results-container::-webkit-scrollbar {
  width: 8px;
}

.results-container::-webkit-scrollbar-track {
  background: transparent;
}

.results-container::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
}

.results-container::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* Utility classes */
.flex {
  display: flex;
}

.flex-col {
  flex-direction: column;
}

.items-center {
  align-items: center;
}

.justify-center {
  justify-content: center;
}

.text-lg {
  font-size: 1.125rem;
}

.text-sm {
  font-size: 0.875rem;
}

.opacity-60 {
  opacity: 0.6;
}

.mt-2 {
  margin-top: 0.5rem;
}

.mb-3 {
  margin-bottom: 0.75rem;
}

.p-6 {
  padding: 1.5rem;
}
</style>
