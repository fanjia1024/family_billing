<template>
  <div class="family-page">
    <div class="header">
      <h1>家庭管理</h1>
      <div class="header-actions">
        <van-button v-if="familyStore.family" type="primary" @click="showAddMemberDialog = true">添加成员</van-button>
        <van-button v-else type="primary" @click="showCreateFamilyDialog = true">创建家庭</van-button>
      </div>
    </div>

    <!-- 空状态 - 引导创建家庭 -->
    <van-empty v-if="!familyStore.family && !familyStore.loading" description="还没有创建家庭">
      <van-button type="primary" @click="showCreateFamilyDialog = true">创建家庭</van-button>
    </van-empty>

    <div class="family-info" v-if="familyStore.family">
      <van-card>
        <template #title>
          <div class="family-title">
            <h2>{{ familyStore.family.name }}</h2>
            <van-icon name="edit" @click="showEditFamilyDialog = true" />
          </div>
        </template>
        <template #desc>
          <p>创建时间: {{ formatDate(familyStore.family.created_at) }}</p>
        </template>
      </van-card>
    </div>

    <div class="members-section">
      <h2>家庭成员</h2>
      <van-list v-if="familyStore.members.length > 0">
        <van-cell
          v-for="member in familyStore.members"
          :key="member.id"
          :title="member.name"
          :label="member.role === 'admin' ? '管理员' : '成员'"
          is-link
          @click="editMember(member)"
        >
          <template #icon>
            <van-icon name="user-o" size="24" style="margin-right: 8px" />
          </template>
          <template #right-icon>
            <van-button
              size="mini"
              type="danger"
              @click.stop="handleDeleteMember(member.id)"
            >
              删除
            </van-button>
          </template>
        </van-cell>
      </van-list>
      <van-empty v-else description="暂无成员" />
    </div>

    <!-- Add Member Dialog -->
    <van-dialog
      v-model:show="showAddMemberDialog"
      title="添加成员"
      show-cancel-button
      @confirm="handleAddMember"
    >
      <van-form @submit="handleAddMember">
        <van-cell-group inset>
          <van-field
            v-model="newMember.name"
            name="name"
            label="姓名"
            placeholder="请输入成员姓名"
            :rules="[{ required: true, message: '请输入成员姓名' }]"
          />
          <van-field
            v-model="newMember.role"
            name="role"
            label="角色"
            placeholder="选择角色"
            is-link
            readonly
            @click="showRolePicker = true"
          />
        </van-cell-group>
      </van-form>
    </van-dialog>

    <!-- Role Picker -->
    <van-popup v-model:show="showRolePicker" position="bottom">
      <van-picker
        :columns="roleColumns"
        @confirm="onRoleConfirm"
        @cancel="showRolePicker = false"
      />
    </van-popup>

    <!-- Create Family Dialog -->
    <van-dialog
      v-model:show="showCreateFamilyDialog"
      title="创建家庭"
      show-cancel-button
      @confirm="handleCreateFamily"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="newFamilyName"
            name="name"
            label="家庭名称"
            placeholder="请输入家庭名称"
            :rules="[{ required: true, message: '请输入家庭名称' }]"
          />
        </van-cell-group>
      </van-form>
    </van-dialog>

    <!-- Edit Family Dialog -->
    <van-dialog
      v-model:show="showEditFamilyDialog"
      title="编辑家庭"
      show-cancel-button
      @confirm="handleUpdateFamily"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="editingFamilyName"
            name="name"
            label="家庭名称"
            placeholder="请输入家庭名称"
            :rules="[{ required: true, message: '请输入家庭名称' }]"
          />
        </van-cell-group>
      </van-form>
    </van-dialog>

    <!-- Edit Member Dialog -->
    <van-dialog
      v-model:show="showEditMemberDialog"
      title="编辑成员"
      show-cancel-button
      @confirm="handleUpdateMember"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="editingMember.name"
            name="name"
            label="姓名"
            placeholder="请输入成员姓名"
            :rules="[{ required: true, message: '请输入成员姓名' }]"
          />
          <van-field
            v-model="editingMember.role"
            name="role"
            label="角色"
            placeholder="选择角色"
            is-link
            readonly
            @click="showEditRolePicker = true"
          />
        </van-cell-group>
      </van-form>
    </van-dialog>

    <!-- Edit Role Picker -->
    <van-popup v-model:show="showEditRolePicker" position="bottom">
      <van-picker
        :columns="roleColumns"
        @confirm="onEditRoleConfirm"
        @cancel="showEditRolePicker = false"
      />
    </van-popup>

    <!-- Category Management Section -->
    <div class="category-section" v-if="familyStore.family">
      <h2>分类管理</h2>
      <CategoryManager />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useFamilyStore } from '../stores/family'
import { showConfirmDialog, showToast } from 'vant'
import CategoryManager from '../components/CategoryManager.vue'
import type { Member } from '../types'

const familyStore = useFamilyStore()
const showAddMemberDialog = ref(false)
const showEditMemberDialog = ref(false)
const showRolePicker = ref(false)
const showEditRolePicker = ref(false)
const showCreateFamilyDialog = ref(false)
const showEditFamilyDialog = ref(false)

const newMember = ref({
  name: '',
  role: 'member' as 'admin' | 'member'
})

const editingMember = ref<Member>({
  id: 0,
  family_id: 0,
  name: '',
  role: 'member',
  created_at: ''
})
const newFamilyName = ref('')
const editingFamilyName = ref('')

const roleColumns = [
  { text: '成员', value: 'member' },
  { text: '管理员', value: 'admin' }
]

const formatDate = (dateStr: string) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN')
}

const onRoleConfirm = ({ selectedOptions }: any) => {
  newMember.value.role = selectedOptions[0].value
  showRolePicker.value = false
}

const onEditRoleConfirm = ({ selectedOptions }: any) => {
  if (editingMember.value) {
    editingMember.value.role = selectedOptions[0].value
  }
  showEditRolePicker.value = false
}

const handleCreateFamily = async () => {
  if (!newFamilyName.value.trim()) {
    showToast('请输入家庭名称')
    return
  }
  
  try {
    await familyStore.createFamily(newFamilyName.value)
    showToast('创建成功')
    showCreateFamilyDialog.value = false
    newFamilyName.value = ''
    // 创建家庭后自动加载成员
    await familyStore.loadMembers()
  } catch (error) {
    showToast('创建失败')
  }
}

const handleUpdateFamily = async () => {
  if (!editingFamilyName.value.trim() || !familyStore.family) {
    showToast('请输入家庭名称')
    return
  }
  
  try {
    await familyStore.updateFamily(familyStore.family.id, editingFamilyName.value)
    showToast('更新成功')
    showEditFamilyDialog.value = false
    editingFamilyName.value = ''
    await familyStore.loadFamily()
  } catch (error) {
    showToast('更新失败')
  }
}

const handleAddMember = async () => {
  if (!newMember.value.name.trim()) {
    showToast('请输入成员姓名')
    return
  }
  
  try {
    await familyStore.createMember(newMember.value.name, newMember.value.role)
    showToast('添加成功')
    showAddMemberDialog.value = false
    newMember.value = { name: '', role: 'member' }
  } catch (error) {
    showToast('添加失败')
  }
}

const editMember = (member: Member) => {
  editingMember.value = { ...member }
  editingFamilyName.value = ''
  showEditMemberDialog.value = true
}

const handleUpdateMember = async () => {
  if (!editingMember.value.name.trim()) {
    showToast('请输入成员姓名')
    return
  }
  
  try {
    await familyStore.updateMember(
      editingMember.value.id,
      editingMember.value.name,
      editingMember.value.role,
      editingMember.value.avatar
    )
    showToast('更新成功')
    showEditMemberDialog.value = false
    editingMember.value = { id: 0, family_id: 0, name: '', role: 'member', created_at: '' }
  } catch (error) {
    showToast('更新失败')
  }
}

const handleDeleteMember = async (id: number) => {
  try {
    await showConfirmDialog({
      title: '确认删除',
      message: '确定要删除这个成员吗？'
    })
    await familyStore.deleteMember(id)
    showToast('删除成功')
  } catch (error) {
    // User cancelled
  }
}

onMounted(async () => {
  await familyStore.loadFamily()
  await familyStore.loadMembers()
  
  // 如果没有家庭，自动创建默认家庭
  if (!familyStore.family && !familyStore.loading) {
    try {
      await familyStore.createFamily('我的家庭')
      await familyStore.loadMembers()
    } catch (error) {
      // 如果创建失败，可能是已存在，重新加载
      await familyStore.loadFamily()
    }
  }
})
</script>

<style scoped>
.family-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.members-section {
  margin-top: 30px;
}

.members-section h2 {
  margin-bottom: 15px;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.family-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.family-title h2 {
  margin: 0;
}

.family-title .van-icon {
  cursor: pointer;
  color: #1989fa;
}

.category-section {
  margin-top: 40px;
}

.category-section h2 {
  margin-bottom: 15px;
}
</style>
