import { invoke } from '@tauri-apps/api/core'
import type {
  HabitsResponse, HabitInput, HabitSummary,
  HeatmapResponse, HabitAnalyticsResponse,
  StreakRewardDto, StreakRewardInput,
  GoalDto, GoalInput, AchievementDto
} from '../types'

export async function fetchHabits(month: number, year: number): Promise<HabitsResponse> {
  return invoke<HabitsResponse>('fetch_habits', { month, year })
}

export async function createHabit(input: HabitInput): Promise<void> {
  return invoke('create_habit', { ...input })
}

export async function updateHabit(input: HabitInput): Promise<void> {
  return invoke('update_habit', { ...input })
}

export async function deleteHabit(id: string): Promise<void> {
  return invoke('delete_habit', { id })
}

export async function toggleHabit(habitId: string, date: string): Promise<void> {
  return invoke('toggle_habit', { habitId, date })
}

export async function fetchHabitSummary(habitId: string): Promise<HabitSummary> {
  return invoke<HabitSummary>('fetch_habit_summary', { habitId })
}

export async function fetchHeatmap(year: number): Promise<HeatmapResponse> {
  return invoke<HeatmapResponse>('fetch_heatmap', { year })
}

export async function fetchHabitAnalytics(): Promise<HabitAnalyticsResponse> {
  return invoke<HabitAnalyticsResponse>('fetch_habit_analytics')
}

export async function fetchRewards(): Promise<StreakRewardDto[]> {
  return invoke<StreakRewardDto[]>('fetch_rewards')
}

export async function createStreakReward(input: StreakRewardInput): Promise<void> {
  return invoke('create_streak_reward', { ...input })
}

export async function updateStreakReward(input: StreakRewardInput): Promise<void> {
  return invoke('update_streak_reward', { ...input })
}

export async function deleteStreakReward(id: string): Promise<void> {
  return invoke('delete_streak_reward', { id })
}

export async function fetchGoals(): Promise<GoalDto[]> {
  return invoke<GoalDto[]>('fetch_goals')
}

export async function createGoal(input: GoalInput): Promise<void> {
  return invoke('create_goal', { ...input })
}

export async function updateGoal(input: GoalInput): Promise<void> {
  return invoke('update_goal', { ...input })
}

export async function deleteGoal(id: string): Promise<void> {
  return invoke('delete_goal', { id })
}

export async function completeGoal(id: string): Promise<void> {
  return invoke('complete_goal', { id })
}

export async function archiveGoal(id: string): Promise<void> {
  return invoke('archive_goal', { id })
}

export async function toggleCheckpoint(id: string): Promise<void> {
  return invoke('toggle_checkpoint', { id })
}

export async function fetchAchievements(): Promise<AchievementDto[]> {
  return invoke<AchievementDto[]>('fetch_achievements')
}
