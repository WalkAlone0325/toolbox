export interface ToolDef {
  id: string
  name: string
  icon: string
  desc: string
  keywords: string[]
}

export type EntryType = "text" | "image" | "files"

export interface ClipboardEntry {
  id: number
  hash: string
  type: EntryType
  text_val: string | null
  image_path: string | null
  thumb_path: string | null
  file_list: string | null
  source_app: string | null
  byte_size: number
  fav: boolean
  pinned: boolean
  created_at: number
  updated_at: number
  last_used_at: number
  use_count: number
  title: string | null
  tags: string | null
  summary: string | null
  note: string | null
}
