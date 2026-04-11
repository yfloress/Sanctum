// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

export interface HabitDto {
  id: string
  name: string
  description: string | null
  color: string
  category: string
  days: boolean[]
}

export interface HabitsResponse {
  habits: HabitDto[]
  month: number
  year: number
  days_in_month: number
}

export interface HabitInput {
  id?: string
  name: string
  description?: string
  color: string
  category: string
}

export interface HabitSummary {
  habit_id: string
  current_streak: number
  best_streak: number
  completion_rate: number
  last_30_days: number
  best_day: string | null
}

export interface HeatmapResponse {
  year: number
  data: HeatmapDay[]
}

export interface HeatmapDay {
  date: string
  intensity: number
}

export interface HabitAnalyticsResponse {
  radar: RadarChartData
  weekday_efficiency: WeekdayChartData
  weekly_summary: string
  insight: string
}

export interface RadarChartData {
  categories: string[]
  values: number[]
  max_value: number
}

export interface WeekdayChartData {
  labels: string[]
  values: number[]
}

export interface StreakRewardDto {
  id: string
  habit_id: string
  habit_name: string
  is_consecutive: boolean
  target_days: number | null
  target_total: number | null
  current_progress: number
  milestones: MilestoneDto[]
}

export interface MilestoneDto {
  id: string
  target_days: number
  reward_text: string
  unlocked: boolean
  unlocked_at: string | null
}

export interface StreakRewardInput {
  id?: string
  habit_id: string
  is_consecutive: boolean
  target_days?: number
  target_total?: number
  milestones: MilestoneInput[]
}

export interface MilestoneInput {
  id?: string
  target_days: number
  reward_text: string
}

export interface GoalDto {
  id: string
  name: string
  description: string | null
  reward_text: string
  deadline: string | null
  is_completed: boolean
  completed_at: string | null
  checkpoints: CheckpointDto[]
}

export interface CheckpointDto {
  id: string
  description: string
  completed: boolean
  completed_at: string | null
}

export interface GoalInput {
  id?: string
  name: string
  description?: string
  reward_text: string
  deadline?: string
  checkpoints: CheckpointInput[]
}

export interface CheckpointInput {
  id?: string
  description: string
}

export interface AchievementDto {
  id: string
  title: string
  description: string
  icon_path: string
  achievement_type: string
  achieved_at: string
}
