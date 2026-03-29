import { invoke } from '@tauri-apps/api/core'
import type {
  HabitsResponse, HabitSummary,
  HeatmapResponse, HabitAnalyticsResponse,
  StreakRewardDto, GoalDto, AchievementDto, MilestoneDto
} from '../types'

export async function fetchHabits(month: number, year: number): Promise<HabitsResponse> {
  return invoke<HabitsResponse>('fetch_habits', { month, year })
}

export async function createHabit(
  name: string, description: string | null, color: string, category: string
): Promise<void> {
  return invoke('create_habit', { name, description, color, category })
}

export async function updateHabit(
  id: string, name: string, description: string | null, color: string, category: string
): Promise<void> {
  return invoke('update_habit', { id, name, description, color, category })
}

export async function deleteHabit(id: string): Promise<void> {
  return invoke('delete_habit', { id })
}

export async function toggleHabit(habit_id: string, date: string): Promise<void> {
  return invoke('toggle_habit', { habit_id, date })
}

export async function fetchHabitSummary(habit_id: string): Promise<HabitSummary> {
  return invoke<HabitSummary>('fetch_habit_summary', { habit_id })
}

export async function fetchHeatmap(year: number): Promise<HeatmapResponse> {
  return invoke<HeatmapResponse>('fetch_heatmap', { year })
}

export async function fetchHabitAnalytics(days?: number): Promise<HabitAnalyticsResponse> {
  return invoke<HabitAnalyticsResponse>('fetch_habit_analytics', { days: days ?? null })
}

export async function fetchRewards(): Promise<StreakRewardDto[]> {
  return invoke<StreakRewardDto[]>('fetch_rewards')
}

export async function createStreakReward(
  habit_id: string, is_consecutive: boolean, target_days: number, target_total: number
): Promise<void> {
  return invoke('create_streak_reward', { habit_id, is_consecutive, target_days, target_total })
}

export async function updateStreakReward(
  id: string, habit_id: string, is_consecutive: boolean,
  target_days: number, target_total: number,
  milestones: [number, string][]
): Promise<void> {
  return invoke('update_streak_reward', { id, habit_id, is_consecutive, target_days, target_total, milestones })
}

export async function deleteStreakReward(id: string): Promise<void> {
  return invoke('delete_streak_reward', { id })
}

export async function addMilestone(reward_id: string, target_days: number, reward_text: string): Promise<MilestoneDto> {
  return invoke<MilestoneDto>('add_milestone', { reward_id, target_days, reward_text })
}

export async function fetchGoals(): Promise<GoalDto[]> {
  return invoke<GoalDto[]>('fetch_goals')
}

export async function createGoal(
  name: string, description: string, reward_text: string, deadline: string
): Promise<void> {
  return invoke('create_goal', { name, description, reward_text, deadline })
}

export async function updateGoal(
  id: string, name: string, description: string, reward_text: string, deadline: string
): Promise<void> {
  return invoke('update_goal', { id, name, description, reward_text, deadline })
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

export async function toggleCheckpoint(goal_id: string, checkpoint_id: string): Promise<void> {
  return invoke('toggle_checkpoint', { goal_id, checkpoint_id })
}

export async function fetchAchievements(): Promise<AchievementDto[]> {
  return invoke<AchievementDto[]>('fetch_achievements')
}
