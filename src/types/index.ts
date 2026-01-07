export interface Family {
  id: number
  name: string
  created_at: string
}

export interface Member {
  id: number
  family_id: number
  name: string
  avatar?: string
  role: 'admin' | 'member'
  created_at: string
}

export interface Category {
  id: number
  name: string
  type: 'income' | 'expense'
  icon?: string
}

export interface Bill {
  id: number
  member_id: number
  category_id: number
  type: 'income' | 'expense'
  amount: number
  description: string
  source: 'wechat' | 'alipay' | 'manual'
  bill_date: string
  bill_month?: string
  created_at: string
  member?: Member
  category?: Category
  images?: BillImage[]
}

export interface BillImage {
  id: number
  bill_id: number
  image_path: string
  ocr_raw_text?: string
  created_at: string
}

export interface OcrResult {
  type: 'income' | 'expense'
  amount: number
  description: string
  bill_date: string
  raw_text: string
}

export interface Statistics {
  total_income: number
  total_expense: number
  balance: number
  monthly_data: MonthlyData[]
  category_data: CategoryData[]
}

export interface MonthlyData {
  month: string
  income: number
  expense: number
}

export interface CategoryData {
  category_id: number
  category_name: string
  amount: number
  percentage: number
}

export interface BillFilters {
  member_id?: number
  category_id?: number
  start_date?: string
  end_date?: string
}

