<template>
  <div class="category-manager">
    <div class="category-header">
      <van-button type="primary" size="small" @click="showAddCategoryDialog = true">添加分类</van-button>
    </div>

    <van-tabs v-model:active="activeTab">
      <van-tab title="收入分类">
        <div class="category-list">
          <van-cell
            v-for="category in incomeCategories"
            :key="category.id"
            :title="category.name"
            :label="category.icon || '无图标'"
            is-link
            @click="editCategory(category)"
          >
            <template #right-icon>
              <van-button
                size="mini"
                type="danger"
                @click.stop="handleDeleteCategory(category.id)"
              >
                删除
              </van-button>
            </template>
          </van-cell>
          <van-empty v-if="incomeCategories.length === 0" description="暂无收入分类" />
        </div>
      </van-tab>
      <van-tab title="支出分类">
        <div class="category-list">
          <van-cell
            v-for="category in expenseCategories"
            :key="category.id"
            :title="category.name"
            :label="category.icon || '无图标'"
            is-link
            @click="editCategory(category)"
          >
            <template #right-icon>
              <van-button
                size="mini"
                type="danger"
                @click.stop="handleDeleteCategory(category.id)"
              >
                删除
              </van-button>
            </template>
          </van-cell>
          <van-empty v-if="expenseCategories.length === 0" description="暂无支出分类" />
        </div>
      </van-tab>
    </van-tabs>

    <!-- Add Category Dialog -->
    <van-dialog
      v-model:show="showAddCategoryDialog"
      title="添加分类"
      show-cancel-button
      @confirm="handleAddCategory"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="newCategory.name"
            name="name"
            label="分类名称"
            placeholder="请输入分类名称"
            :rules="[{ required: true, message: '请输入分类名称' }]"
          />
          <van-field
            v-model="newCategory.type"
            name="type"
            label="类型"
            placeholder="选择类型"
            is-link
            readonly
            @click="showTypePicker = true"
          />
          <van-field
            v-model="newCategory.icon"
            name="icon"
            label="图标"
            placeholder="输入图标名称（可选）"
          />
        </van-cell-group>
      </van-form>
    </van-dialog>

    <!-- Edit Category Dialog -->
    <van-dialog
      v-model:show="showEditCategoryDialog"
      title="编辑分类"
      show-cancel-button
      @confirm="handleUpdateCategory"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="editingCategory.name"
            name="name"
            label="分类名称"
            placeholder="请输入分类名称"
            :rules="[{ required: true, message: '请输入分类名称' }]"
          />
          <van-field
            v-model="editingCategory.icon"
            name="icon"
            label="图标"
            placeholder="输入图标名称（可选）"
          />
        </van-cell-group>
      </van-form>
    </van-dialog>

    <!-- Type Picker -->
    <van-popup v-model:show="showTypePicker" position="bottom">
      <van-picker
        :columns="typeColumns"
        @confirm="onTypeConfirm"
        @cancel="showTypePicker = false"
      />
    </van-popup>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useBillStore } from '../stores/bill'
import { showConfirmDialog, showToast } from 'vant'
import type { Category } from '../types'

const billStore = useBillStore()
const activeTab = ref(0)
const showAddCategoryDialog = ref(false)
const showEditCategoryDialog = ref(false)
const showTypePicker = ref(false)

const newCategory = ref({
  name: '',
  type: 'expense' as 'income' | 'expense',
  icon: ''
})

const editingCategory = ref<Category>({
  id: 0,
  name: '',
  type: 'expense',
  icon: ''
})

const typeColumns = [
  { text: '支出', value: 'expense' },
  { text: '收入', value: 'income' }
]

const incomeCategories = computed(() => {
  return billStore.categories.filter(c => c.type === 'income')
})

const expenseCategories = computed(() => {
  return billStore.categories.filter(c => c.type === 'expense')
})

const onTypeConfirm = ({ selectedOptions }: any) => {
  newCategory.value.type = selectedOptions[0].value
  showTypePicker.value = false
}

const handleAddCategory = async () => {
  if (!newCategory.value.name.trim()) {
    showToast('请输入分类名称')
    return
  }
  
  try {
    await billStore.createCategory(
      newCategory.value.name,
      newCategory.value.type,
      newCategory.value.icon || undefined
    )
    showToast('添加成功')
    showAddCategoryDialog.value = false
    newCategory.value = { name: '', type: 'expense', icon: '' }
  } catch (error) {
    showToast('添加失败')
  }
}

const editCategory = (category: Category) => {
  editingCategory.value = { ...category }
  showEditCategoryDialog.value = true
}

const handleUpdateCategory = async () => {
  if (!editingCategory.value.name.trim()) {
    showToast('请输入分类名称')
    return
  }
  
  try {
    await billStore.updateCategory(
      editingCategory.value.id,
      editingCategory.value.name,
      editingCategory.value.icon || undefined
    )
    showToast('更新成功')
    showEditCategoryDialog.value = false
    editingCategory.value = { id: 0, name: '', type: 'expense', icon: '' }
  } catch (error) {
    showToast('更新失败')
  }
}

const handleDeleteCategory = async (id: number) => {
  try {
    await showConfirmDialog({
      title: '确认删除',
      message: '确定要删除这个分类吗？删除后相关账单的分类将无法显示。'
    })
    await billStore.deleteCategory(id)
    showToast('删除成功')
  } catch (error) {
    // User cancelled
  }
}

onMounted(async () => {
  await billStore.loadCategories()
})
</script>

<style scoped>
.category-manager {
  margin-top: 20px;
}

.category-header {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 15px;
}

.category-list {
  margin-top: 15px;
}
</style>

