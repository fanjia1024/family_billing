<template>
  <div class="bills-page">
    <div class="header">
      <h1>账单管理</h1>
      <div class="actions">
        <van-button type="primary" @click="showAddBillDialog = true">添加账单</van-button>
        <van-button type="success" @click="showOcrDialog = true">OCR识别</van-button>
      </div>
    </div>

    <div class="filters">
      <van-field
        v-model="filters.member_id"
        label="成员"
        placeholder="选择成员"
        is-link
        readonly
        @click="showMemberPicker = true"
      />
      <van-field
        v-model="filters.category_id"
        label="分类"
        placeholder="选择分类"
        is-link
        readonly
        @click="showCategoryPicker = true"
      />
      <van-button type="primary" @click="loadBills">筛选</van-button>
      <van-button @click="resetFilters">重置</van-button>
    </div>

    <div v-if="billStore.bills.length > 0" class="bills-list">
      <BillCard
        v-for="bill in billStore.bills"
        :key="bill.id"
        :bill="bill"
        @edit="viewBill"
        @delete="handleDeleteBill"
      />
    </div>
    <van-empty v-else description="暂无账单" />

    <!-- Add Bill Dialog -->
    <van-dialog
      v-model:show="showAddBillDialog"
      :title="editingBill ? '编辑账单' : '添加账单'"
      show-cancel-button
      @confirm="handleAddBill"
      @close="resetBillForm"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="newBill.amount"
            name="amount"
            label="金额"
            type="number"
            placeholder="请输入金额"
            :rules="[{ required: true, message: '请输入金额' }]"
          />
          <van-field
            v-model="newBill.description"
            name="description"
            label="描述"
            placeholder="请输入描述"
          />
          <van-field
            v-model="memberDisplayName"
            name="member"
            label="成员"
            placeholder="选择成员"
            is-link
            readonly
            @click="showNewBillMemberPicker = true"
          />
          <van-field
            v-model="categoryDisplayName"
            name="category"
            label="分类"
            placeholder="选择分类"
            is-link
            readonly
            @click="showNewBillCategoryPicker = true"
          />
          <van-field
            v-model="newBill.type"
            name="type"
            label="类型"
            placeholder="选择类型"
            is-link
            readonly
            @click="showTypePicker = true"
          />
          <van-field
            v-model="sourceDisplayName"
            name="source"
            label="来源"
            placeholder="选择来源"
            is-link
            readonly
            @click="showSourcePicker = true"
          />
          <van-field
            v-model="newBill.bill_date"
            name="bill_date"
            label="日期"
            placeholder="选择日期"
            is-link
            readonly
            @click="showDatePicker = true"
          />
        </van-cell-group>
      </van-form>
    </van-dialog>

    <!-- Edit Bill Dialog (复用 Add Bill Dialog，通过 editingBill 状态区分) -->

    <!-- OCR Dialog -->
    <van-dialog
      v-model:show="showOcrDialog"
      title="OCR识别账单"
      show-cancel-button
      @confirm="handleConfirmOcrBill"
      @cancel="resetOcrDialog"
    >
      <div class="ocr-upload-section">
        <van-uploader
          v-model="fileList"
          :after-read="handleOcrUpload"
          accept="image/*"
          :max-count="1"
        />
        <p class="ocr-tip">支持微信、支付宝账单截图</p>
      </div>
      <div v-if="ocrResult" class="ocr-result">
        <h3>识别结果</h3>
        <van-cell-group inset>
          <van-field
            v-model.number="ocrResult.amount"
            label="金额"
            type="number"
            placeholder="识别金额"
          />
          <van-field
            v-model="ocrResult.type"
            label="类型"
            placeholder="选择类型"
            is-link
            readonly
            @click="showOcrTypePicker = true"
          />
          <van-field
            v-model="ocrResult.bill_date"
            label="日期"
            placeholder="选择日期"
            is-link
            readonly
            @click="showOcrDatePicker = true"
          />
          <van-field
            v-model="ocrResult.description"
            label="描述"
            placeholder="账单描述"
            type="textarea"
            rows="2"
          />
        </van-cell-group>
        <div class="ocr-raw-text" v-if="ocrResult.raw_text">
          <p><strong>原始文本:</strong></p>
          <p class="raw-text">{{ ocrResult.raw_text }}</p>
        </div>
      </div>
    </van-dialog>

    <!-- Date Picker -->
    <van-popup v-model:show="showDatePicker" position="bottom">
      <van-date-picker
        v-model="datePickerValue"
        @confirm="onDateConfirm"
        @cancel="showDatePicker = false"
      />
    </van-popup>

    <!-- OCR Date Picker -->
    <van-popup v-model:show="showOcrDatePicker" position="bottom">
      <van-date-picker
        v-model="ocrDatePickerValue"
        @confirm="onOcrDateConfirm"
        @cancel="showOcrDatePicker = false"
      />
    </van-popup>

    <!-- Type Picker -->
    <van-popup v-model:show="showTypePicker" position="bottom">
      <van-picker
        :columns="typeColumns"
        @confirm="onTypeConfirm"
        @cancel="showTypePicker = false"
      />
    </van-popup>

    <!-- OCR Type Picker -->
    <van-popup v-model:show="showOcrTypePicker" position="bottom">
      <van-picker
        :columns="typeColumns"
        @confirm="onOcrTypeConfirm"
        @cancel="showOcrTypePicker = false"
      />
    </van-popup>

    <!-- Source Picker -->
    <van-popup v-model:show="showSourcePicker" position="bottom">
      <van-picker
        :columns="sourceColumns"
        @confirm="onSourceConfirm"
        @cancel="showSourcePicker = false"
      />
    </van-popup>

    <!-- Member Picker (for filters) -->
    <van-popup v-model:show="showMemberPicker" position="bottom">
      <van-picker
        :columns="memberColumns"
        @confirm="onFilterMemberConfirm"
        @cancel="showMemberPicker = false"
      />
    </van-popup>

    <!-- Category Picker (for filters) -->
    <van-popup v-model:show="showCategoryPicker" position="bottom">
      <van-picker
        :columns="categoryColumns"
        @confirm="onFilterCategoryConfirm"
        @cancel="showCategoryPicker = false"
      />
    </van-popup>

    <!-- Member Picker (for new bill) -->
    <van-popup v-model:show="showNewBillMemberPicker" position="bottom">
      <van-picker
        :columns="memberColumns"
        @confirm="onNewBillMemberConfirm"
        @cancel="showNewBillMemberPicker = false"
      />
    </van-popup>

    <!-- Category Picker (for new bill) -->
    <van-popup v-model:show="showNewBillCategoryPicker" position="bottom">
      <van-picker
        :columns="categoryColumns"
        @confirm="onNewBillCategoryConfirm"
        @cancel="showNewBillCategoryPicker = false"
      />
    </van-popup>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useBillStore } from '../stores/bill'
import { useFamilyStore } from '../stores/family'
import { ocrApi, imageApi } from '../api/tauri'
import { showConfirmDialog, showToast } from 'vant'
import { validateBill } from '../utils/validation'
import { formatErrorMessage } from '../utils/errorHandler'
import BillCard from '../components/BillCard.vue'
import type { Bill } from '../types'

const billStore = useBillStore()
const familyStore = useFamilyStore()
const showAddBillDialog = ref(false)
const showOcrDialog = ref(false)
const showMemberPicker = ref(false)
const showCategoryPicker = ref(false)
const showTypePicker = ref(false)
const showDatePicker = ref(false)
const showOcrTypePicker = ref(false)
const showOcrDatePicker = ref(false)
const showSourcePicker = ref(false)
const showNewBillMemberPicker = ref(false)
const showNewBillCategoryPicker = ref(false)
const fileList = ref([])
const ocrResult = ref<any>(null)
const currentImagePath = ref<string | null>(null)
const editingBill = ref<Bill | null>(null)

// Vant 4 DatePicker 需要字符串数组格式 ['2024', '01', '05']
const today = new Date()
const datePickerValue = ref<string[]>([
  String(today.getFullYear()),
  String(today.getMonth() + 1).padStart(2, '0'),
  String(today.getDate()).padStart(2, '0')
])
const ocrDatePickerValue = ref<string[]>([
  String(today.getFullYear()),
  String(today.getMonth() + 1).padStart(2, '0'),
  String(today.getDate()).padStart(2, '0')
])

const filters = ref({
  member_id: null as number | null,
  category_id: null as number | null
})

const memberDisplayName = ref('')
const categoryDisplayName = ref('')
const sourceDisplayName = ref('')

const newBill = ref({
  member_id: 0,
  category_id: 0,
  type: 'expense' as 'income' | 'expense',
  amount: 0,
  description: '',
  source: 'manual' as 'wechat' | 'alipay' | 'manual',
  bill_date: new Date().toISOString().split('T')[0]
})

const typeColumns = [
  { text: '支出', value: 'expense' },
  { text: '收入', value: 'income' }
]

const sourceColumns = [
  { text: '手动', value: 'manual' },
  { text: '微信', value: 'wechat' },
  { text: '支付宝', value: 'alipay' }
]

const memberColumns = computed(() => {
  return [
    { text: '全部', value: null },
    ...familyStore.members.map(m => ({ text: m.name, value: m.id }))
  ]
})

const categoryColumns = computed(() => {
  return [
    { text: '全部', value: null },
    ...billStore.categories.map(c => ({ text: c.name, value: c.id }))
  ]
})

const loadBills = async () => {
  await billStore.loadBills({
    member_id: filters.value.member_id || undefined,
    category_id: filters.value.category_id || undefined
  })
}

const resetFilters = () => {
  filters.value = { member_id: null, category_id: null }
  loadBills()
}

const handleAddBill = async () => {
  // 1. 先设置默认值（如果未选择）
  if (!newBill.value.member_id || newBill.value.member_id <= 0) {
    if (familyStore.members.length > 0) {
      newBill.value.member_id = familyStore.members[0].id
    } else {
      showToast('请先添加家庭成员')
      return
    }
  }
  
  if (!newBill.value.category_id || newBill.value.category_id <= 0) {
    const defaultCategory = billStore.categories.find(
      c => c.type === newBill.value.type
    )
    if (defaultCategory) {
      newBill.value.category_id = defaultCategory.id
    } else if (billStore.categories.length > 0) {
      newBill.value.category_id = billStore.categories[0].id
    } else {
      showToast('请先加载分类')
      return
    }
  }
  
  // 2. 确保所有字段都有有效值
  // 确保 description 不是 undefined 或 null
  if (newBill.value.description === undefined || newBill.value.description === null) {
    newBill.value.description = ''
  }
  
  // 确保 bill_date 存在
  if (!newBill.value.bill_date) {
    newBill.value.bill_date = new Date().toISOString().split('T')[0]
  }
  
  // 确保 source 存在
  if (!newBill.value.source) {
    newBill.value.source = 'manual'
  }
  
  // 3. 再进行验证
  const validation = validateBill({
    member_id: newBill.value.member_id,
    category_id: newBill.value.category_id,
    amount: newBill.value.amount,
    bill_date: newBill.value.bill_date,
    description: newBill.value.description || ''
  })
  
  if (!validation.valid) {
    showToast(validation.message || '数据验证失败')
    return
  }
  
  // 4. 准备提交的数据，确保所有字段都有值
  const billData = {
    member_id: newBill.value.member_id,
    category_id: newBill.value.category_id,
    type: newBill.value.type,
    amount: Number(newBill.value.amount),
    description: newBill.value.description || '',
    source: newBill.value.source,
    bill_date: newBill.value.bill_date
  }
  
  // 调试日志
  console.log('提交账单数据:', billData)
  
  try {
    if (editingBill.value) {
      // Update existing bill
      await billStore.updateBill(editingBill.value.id, {
        member_id: billData.member_id,
        category_id: billData.category_id,
        type: billData.type,
        amount: billData.amount,
        description: billData.description,
        bill_date: billData.bill_date
      })
      showToast('更新成功')
    } else {
      // Create new bill
      await billStore.createBill(billData)
      showToast('添加成功')
    }
    showAddBillDialog.value = false
    resetBillForm()
    await loadBills()
  } catch (error) {
    console.error('Save bill error:', error)
    const errorMessage = formatErrorMessage(error, editingBill.value ? '更新失败' : '添加失败')
    showToast(errorMessage)
  }
}

const handleOcrUpload = async (file: any) => {
  try {
    if (!file.file) {
      showToast('请选择图片文件')
      return
    }
    
    showToast('正在识别...')
    
    // Convert file to base64 or array buffer
    const reader = new FileReader()
    reader.onload = async (e) => {
      try {
        const arrayBuffer = e.target?.result as ArrayBuffer
        const uint8Array = new Uint8Array(arrayBuffer)
        const imageData = Array.from(uint8Array)
        
        // Save image to backend
        const imagePath = await imageApi.saveUploadedImage(imageData, file.file.name)
        currentImagePath.value = imagePath
        
        // Perform OCR recognition
        const result = await ocrApi.recognizeImage(imagePath)
        ocrResult.value = result
        
        showToast('OCR识别完成，请确认信息')
      } catch (error) {
        console.error('OCR error:', error)
        showToast('OCR识别失败: ' + (error as Error).message)
      }
    }
    
    reader.onerror = () => {
      showToast('图片读取失败')
    }
    
    reader.readAsArrayBuffer(file.file)
  } catch (error) {
    console.error('Upload error:', error)
    showToast('上传失败')
  }
}

const handleConfirmOcrBill = async () => {
  if (!ocrResult.value) {
    showToast('请先上传图片并识别')
    return
  }
  
  if (!ocrResult.value.amount || ocrResult.value.amount <= 0) {
    showToast('请输入有效金额')
    return
  }
  
  if (!familyStore.members.length) {
    showToast('请先添加家庭成员')
    return
  }
  
  if (!billStore.categories.length) {
    showToast('请先加载分类')
    return
  }
  
  try {
    // Get default member and category
    const defaultMember = familyStore.members[0]
    const defaultCategory = billStore.categories.find(
      c => c.type === ocrResult.value.type
    ) || billStore.categories[0]
    
    const bill = {
      member_id: defaultMember.id,
      category_id: defaultCategory.id,
      type: ocrResult.value.type as 'income' | 'expense',
      amount: ocrResult.value.amount,
      description: ocrResult.value.description || 'OCR识别账单',
      source: 'wechat' as 'wechat' | 'alipay' | 'manual',
      bill_date: ocrResult.value.bill_date || new Date().toISOString().split('T')[0]
    }
    
    if (currentImagePath.value) {
      await ocrApi.saveBillWithImage(bill, currentImagePath.value)
    } else {
      await billStore.createBill(bill)
    }
    
    showToast('账单保存成功')
    showOcrDialog.value = false
    resetOcrDialog()
    await loadBills()
  } catch (error) {
    console.error('Save bill error:', error)
    showToast('保存失败')
  }
}

const resetOcrDialog = () => {
  fileList.value = []
  ocrResult.value = null
  currentImagePath.value = null
}

const viewBill = (bill: Bill) => {
  editingBill.value = bill
  newBill.value = {
    member_id: bill.member_id,
    category_id: bill.category_id,
    type: bill.type,
    amount: bill.amount,
    description: bill.description,
    source: bill.source,
    bill_date: bill.bill_date
  }
  
  // 更新显示名称
  const member = familyStore.members.find(m => m.id === bill.member_id)
  memberDisplayName.value = member ? member.name : ''
  
  const category = billStore.categories.find(c => c.id === bill.category_id)
  categoryDisplayName.value = category ? category.name : ''
  
  // 设置来源显示名称
  const sourceOption = sourceColumns.find(s => s.value === bill.source)
  sourceDisplayName.value = sourceOption ? sourceOption.text : ''
  
  // 设置日期选择器值 - 转换为字符串数组
  const dateParts = bill.bill_date.split('-')
  if (dateParts.length === 3) {
    datePickerValue.value = dateParts
  }
  
  showAddBillDialog.value = true
}

const resetBillForm = () => {
  editingBill.value = null
  const today = new Date()
  newBill.value = {
    member_id: 0,
    category_id: 0,
    type: 'expense',
    amount: 0,
    description: '',
    source: 'manual',
    bill_date: today.toISOString().split('T')[0]
  }
  memberDisplayName.value = ''
  categoryDisplayName.value = ''
  // 设置默认来源显示名称
  const defaultSource = sourceColumns.find(s => s.value === 'manual')
  sourceDisplayName.value = defaultSource ? defaultSource.text : ''
  datePickerValue.value = [
    String(today.getFullYear()),
    String(today.getMonth() + 1).padStart(2, '0'),
    String(today.getDate()).padStart(2, '0')
  ]
}

// Date picker handlers - Vant 4 返回 { selectedValues: ['2024', '01', '05'] }
const onDateConfirm = ({ selectedValues }: { selectedValues: string[] }) => {
  newBill.value.bill_date = selectedValues.join('-')
  datePickerValue.value = selectedValues
  showDatePicker.value = false
}

const onOcrDateConfirm = ({ selectedValues }: { selectedValues: string[] }) => {
  if (ocrResult.value) {
    ocrResult.value.bill_date = selectedValues.join('-')
  }
  ocrDatePickerValue.value = selectedValues
  showOcrDatePicker.value = false
}

// Type picker handlers
const onTypeConfirm = ({ selectedOptions }: any) => {
  const newType = selectedOptions[0].value
  newBill.value.type = newType
  showTypePicker.value = false
  
  // 智能重置分类：只在当前分类与新类型不匹配时才重置
  if (newBill.value.category_id > 0) {
    const currentCategory = billStore.categories.find(c => c.id === newBill.value.category_id)
    if (currentCategory && currentCategory.type !== newType) {
      // 分类类型不匹配，需要重置
      categoryDisplayName.value = ''
      newBill.value.category_id = 0
    }
    // 如果分类类型匹配，保留当前选择
  }
}

const onOcrTypeConfirm = ({ selectedOptions }: any) => {
  if (ocrResult.value) {
    ocrResult.value.type = selectedOptions[0].value
  }
  showOcrTypePicker.value = false
}

// Source picker handler
const onSourceConfirm = ({ selectedOptions }: any) => {
  const sourceValue = selectedOptions[0].value
  newBill.value.source = sourceValue
  sourceDisplayName.value = selectedOptions[0].text
  showSourcePicker.value = false
}

// Filter picker handlers
const onFilterMemberConfirm = ({ selectedOptions }: any) => {
  filters.value.member_id = selectedOptions[0].value
  showMemberPicker.value = false
}

const onFilterCategoryConfirm = ({ selectedOptions }: any) => {
  filters.value.category_id = selectedOptions[0].value
  showCategoryPicker.value = false
}

// New bill picker handlers
const onNewBillMemberConfirm = ({ selectedOptions }: any) => {
  const memberId = selectedOptions[0].value
  if (memberId) {
    newBill.value.member_id = memberId
    memberDisplayName.value = selectedOptions[0].text
  }
  showNewBillMemberPicker.value = false
}

const onNewBillCategoryConfirm = ({ selectedOptions }: any) => {
  const categoryId = selectedOptions[0].value
  if (categoryId) {
    newBill.value.category_id = categoryId
    categoryDisplayName.value = selectedOptions[0].text
  }
  showNewBillCategoryPicker.value = false
}

const handleDeleteBill = async (id: number) => {
  try {
    await showConfirmDialog({
      title: '确认删除',
      message: '确定要删除这条账单吗？'
    })
    await billStore.deleteBill(id)
    showToast('删除成功')
  } catch (error) {
    // User cancelled
  }
}

onMounted(async () => {
  await familyStore.loadMembers()
  await billStore.loadCategories()
  await loadBills()
})
</script>

<style scoped>
.bills-page {
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

.actions {
  display: flex;
  gap: 10px;
}

.filters {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.ocr-upload-section {
  padding: 15px;
}

.ocr-tip {
  margin-top: 10px;
  font-size: 12px;
  color: #999;
  text-align: center;
}

.ocr-result {
  margin-top: 20px;
  padding: 15px;
}

.ocr-raw-text {
  margin-top: 15px;
  padding: 10px;
  background: #f5f5f5;
  border-radius: 4px;
  font-size: 12px;
}

.raw-text {
  color: #666;
  word-break: break-all;
  white-space: pre-wrap;
}

.bills-list {
  margin-top: 20px;
  display: flex;
  flex-direction: column;
  gap: 15px;
}
</style>
