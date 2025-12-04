/**
 * HabitsView - Atomic Habits Tracker (OPTIMIZED)
 *
 * Monthly grid tracker with cyberpunk/neon styling.
 * - Rows: Habits
 * - Columns: Days of the month (1-31)
 * - Completed cells glow with the habit's color
 *
 * PERFORMANCE OPTIMIZATIONS:
 * - HabitCell and HabitRow are memoized with React.memo
 * - Stats are pre-computed in the store (O(1) lookups)
 * - Chart data is memoized with useMemo
 * - Callbacks are memoized with useCallback
 */

import { memo, useCallback, useEffect, useMemo, useState } from "react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  useCurrentMonth,
  useHabitForm,
  useHabitLoading,
  useHabits,
  useHabitStats,
  useHabitStore,
  useHabitToDelete,
  useHabitToEdit,
  useShowAddModal,
} from "../../stores/index.ts";
import { HABIT_COLORS } from "../../types/index.ts";
import { DeleteConfirmModal } from "../../components/modals/DeleteConfirmModal.tsx";

// ==================== Helper Functions ====================

const getDaysInMonth = (year: number, month: number): number => {
  return new Date(year, month + 1, 0).getDate();
};

const formatDateToISO = (year: number, month: number, day: number): string => {
  return `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(
    2,
    "0",
  )}`;
};

const getMonthName = (date: Date): string => {
  return date.toLocaleDateString("en-US", { month: "long", year: "numeric" });
};

const isToday = (year: number, month: number, day: number): boolean => {
  const today = new Date();
  return (
    today.getFullYear() === year &&
    today.getMonth() === month &&
    today.getDate() === day
  );
};

const isFutureDate = (year: number, month: number, day: number): boolean => {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const checkDate = new Date(year, month, day);
  return checkDate > today;
};

// ==================== Memoized Sub-Components ====================

interface HabitCellProps {
  habitColor: string;
  isCompleted: boolean;
  isFuture: boolean;
  isToday: boolean;
  onToggle: () => void;
}

const HabitCell = memo(function HabitCell({
  habitColor,
  isCompleted,
  isFuture,
  isToday: isTodayCell,
  onToggle,
}: HabitCellProps) {
  const handleClick = () => {
    if (!isFuture) {
      onToggle();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.key === "Enter" || e.key === " ") && !isFuture) {
      e.preventDefault();
      onToggle();
    }
  };

  return (
    <div
      className={`habit-cell ${isCompleted ? "completed" : ""} ${
        isFuture ? "future" : ""
      } ${isTodayCell ? "today" : ""}`}
      style={
        isCompleted
          ? {
              backgroundColor: habitColor,
              boxShadow: `0 0 12px ${habitColor}80, 0 0 24px ${habitColor}40`,
            }
          : undefined
      }
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={isFuture ? -1 : 0}
      role="checkbox"
      aria-checked={isCompleted}
      aria-disabled={isFuture}
    />
  );
});

interface HabitRowProps {
  habitId: string;
  habitName: string;
  habitColor: string;
  year: number;
  month: number;
  daysInMonth: number;
  completionRate: number;
  streak: number;
  onEdit: () => void;
  onDelete: () => void;
}

const HabitRow = memo(function HabitRow({
  habitId,
  habitName,
  habitColor,
  year,
  month,
  daysInMonth,
  completionRate,
  streak,
  onEdit,
  onDelete,
}: HabitRowProps) {
  const [isHovered, setIsHovered] = useState(false);

  // Get these from the store directly in the row to avoid prop drilling
  const isCompleted = useHabitStore((state) => state.isCompleted);
  const toggleLog = useHabitStore((state) => state.toggleLog);

  // Memoize the toggle handler for each day
  const handleToggle = useCallback(
    (date: string) => {
      toggleLog(habitId, date);
    },
    [habitId, toggleLog],
  );

  return (
    <div
      className={`habit-row ${isHovered ? "hovered" : ""}`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Habit Info Column */}
      <div className="habit-info">
        <div
          className="habit-color-indicator"
          style={{ backgroundColor: habitColor }}
        />
        <div className="habit-details">
          <span className="habit-name">{habitName}</span>
          <div className="habit-stats">
            <span className="habit-rate">{completionRate}%</span>
            {streak > 0 && <span className="habit-streak">🔥 {streak}</span>}
          </div>
        </div>
        <div className="habit-actions">
          <button
            type="button"
            className="habit-action-btn edit"
            onClick={onEdit}
            title="Edit habit"
          >
            ✏️
          </button>
          <button
            type="button"
            className="habit-action-btn delete"
            onClick={onDelete}
            title="Delete habit"
          >
            🗑️
          </button>
        </div>
      </div>

      {/* Days Grid */}
      <div className="habit-days">
        {Array.from({ length: daysInMonth }, (_, i) => {
          const day = i + 1;
          const date = formatDateToISO(year, month, day);
          const future = isFutureDate(year, month, day);
          const todayCell = isToday(year, month, day);
          const completed = isCompleted(habitId, date);

          return (
            <HabitCell
              key={day}
              habitColor={habitColor}
              isCompleted={completed}
              isFuture={future}
              isToday={todayCell}
              onToggle={() => handleToggle(date)}
            />
          );
        })}
      </div>
    </div>
  );
});

interface AddHabitModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: () => void;
  form: { name: string; description: string; color: string };
  setFormField: (
    field: "name" | "description" | "color",
    value: string,
  ) => void;
  isLoading: boolean;
  isEdit: boolean;
}

const AddHabitModal = memo(function AddHabitModal({
  isOpen,
  onClose,
  onSubmit,
  form,
  setFormField,
  isLoading,
  isEdit,
}: AddHabitModalProps) {
  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit();
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-card habit-modal"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <div className="modal-icon">🎯</div>
          <h2>{isEdit ? "Edit Habit" : "New Habit"}</h2>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            <div className="form-group">
              <label htmlFor="habit-name">Name</label>
              <input
                id="habit-name"
                type="text"
                value={form.name}
                onChange={(e) => setFormField("name", e.target.value)}
                placeholder="e.g., Meditate, Exercise, Read..."
                autoFocus
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="habit-description">Description (optional)</label>
              <input
                id="habit-description"
                type="text"
                value={form.description}
                onChange={(e) => setFormField("description", e.target.value)}
                placeholder="Brief description or goal..."
              />
            </div>

            <div className="form-group">
              <label>Color</label>
              <div className="color-picker">
                {HABIT_COLORS.map((color) => (
                  <button
                    key={color}
                    type="button"
                    className={`color-option ${
                      form.color === color ? "selected" : ""
                    }`}
                    style={{ backgroundColor: color }}
                    onClick={() => setFormField("color", color)}
                    aria-label={`Select color ${color}`}
                  />
                ))}
              </div>
            </div>
          </div>

          <div className="modal-actions">
            <button
              type="button"
              className="btn-secondary"
              onClick={onClose}
              disabled={isLoading}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="btn-primary"
              disabled={isLoading || !form.name.trim()}
            >
              {isLoading ? "Saving..." : isEdit ? "Update" : "Create"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
});

// ==================== Main Component ====================

export const HabitsView = () => {
  // Store State (using optimized selectors)
  const habits = useHabits();
  const currentMonth = useCurrentMonth();
  const isLoading = useHabitLoading();
  const showAddModal = useShowAddModal();
  const habitToEdit = useHabitToEdit();
  const habitToDelete = useHabitToDelete();
  const form = useHabitForm();
  const stats = useHabitStats(); // Pre-computed stats from store

  // Store Actions
  const loadAll = useHabitStore((state) => state.loadAll);
  const goToPreviousMonth = useHabitStore((state) => state.goToPreviousMonth);
  const goToNextMonth = useHabitStore((state) => state.goToNextMonth);
  const goToToday = useHabitStore((state) => state.goToToday);
  const setShowAddModal = useHabitStore((state) => state.setShowAddModal);
  const setHabitToEdit = useHabitStore((state) => state.setHabitToEdit);
  const setHabitToDelete = useHabitStore((state) => state.setHabitToDelete);
  const setFormField = useHabitStore((state) => state.setFormField);
  const addHabit = useHabitStore((state) => state.addHabit);
  const updateHabit = useHabitStore((state) => state.updateHabit);
  const confirmDelete = useHabitStore((state) => state.confirmDelete);
  const cancelDelete = useHabitStore((state) => state.cancelDelete);

  // Local State
  const [hasLoaded, setHasLoaded] = useState(false);

  // Load data on mount
  useEffect(() => {
    if (!hasLoaded) {
      loadAll();
      setHasLoaded(true);
    }
  }, [loadAll, hasLoaded]);

  // Derived values (memoized)
  const year = currentMonth.getFullYear();
  const month = currentMonth.getMonth();
  const daysInMonth = useMemo(() => getDaysInMonth(year, month), [year, month]);
  const monthName = useMemo(() => getMonthName(currentMonth), [currentMonth]);

  const maxPossibleCompletions = useMemo(() => {
    const today = new Date();
    const isCurrentMonth =
      today.getFullYear() === year && today.getMonth() === month;
    const activeDays = isCurrentMonth ? today.getDate() : daysInMonth;
    return habits.length * activeDays;
  }, [habits.length, daysInMonth, year, month]);

  const overallProgress = useMemo(() => {
    if (maxPossibleCompletions === 0) return 0;
    return Math.round((stats.totalCompletions / maxPossibleCompletions) * 100);
  }, [stats.totalCompletions, maxPossibleCompletions]);

  // Chart data (memoized, uses cached stats)
  const chartData = useMemo(() => {
    const data: { day: string; completions: number }[] = [];

    for (let day = 1; day <= daysInMonth; day++) {
      const date = formatDateToISO(year, month, day);
      const completions = stats.completionsPerDay.get(date) || 0;

      // Only include days up to today for current month
      const future = isFutureDate(year, month, day);
      if (!future || completions > 0) {
        data.push({
          day: String(day),
          completions,
        });
      }
    }

    return data;
  }, [daysInMonth, year, month, stats.completionsPerDay]);

  // Handlers (memoized)
  const handleAddSubmit = useCallback(async () => {
    if (habitToEdit) {
      await updateHabit(habitToEdit.id, form);
    } else {
      await addHabit(form);
    }
  }, [habitToEdit, form, addHabit, updateHabit]);

  const handleCloseModal = useCallback(() => {
    setShowAddModal(false);
    setHabitToEdit(null);
  }, [setShowAddModal, setHabitToEdit]);

  // Check if we're viewing current month
  const today = new Date();
  const isCurrentMonthView =
    today.getFullYear() === year && today.getMonth() === month;

  return (
    <div className="habits-page">
      {/* Header */}
      <div className="habits-header">
        <div className="habits-title-section">
          <h1 className="page-title">🎯 Atomic Habits</h1>
          <p className="habits-subtitle">
            Build better habits, one day at a time
          </p>
        </div>

        <button
          type="button"
          className="btn-primary add-habit-btn"
          onClick={() => setShowAddModal(true)}
        >
          <span>+</span> New Habit
        </button>
      </div>

      {/* Month Navigation */}
      <div className="habits-nav">
        <div className="month-selector">
          <button
            type="button"
            className="nav-arrow"
            onClick={goToPreviousMonth}
            aria-label="Previous month"
          >
            ←
          </button>
          <h2 className="current-month">{monthName}</h2>
          <button
            type="button"
            className="nav-arrow"
            onClick={goToNextMonth}
            aria-label="Next month"
          >
            →
          </button>
        </div>

        <div className="nav-actions">
          {!isCurrentMonthView && (
            <button
              type="button"
              className="btn-secondary today-btn"
              onClick={goToToday}
            >
              Today
            </button>
          )}
        </div>

        <div className="month-progress">
          <div className="progress-label">
            <span>Monthly Progress</span>
            <span className="progress-value">{overallProgress}%</span>
          </div>
          <div className="progress-bar">
            <div
              className="progress-fill"
              style={{ width: `${overallProgress}%` }}
            />
          </div>
          <span className="progress-detail">
            {stats.totalCompletions} / {maxPossibleCompletions} completions
          </span>
        </div>
      </div>

      {/* Loading State */}
      {isLoading && habits.length === 0 && (
        <div className="habits-loading">
          <div className="loader" />
          <p>Loading habits...</p>
        </div>
      )}

      {/* Empty State */}
      {!isLoading && habits.length === 0 && (
        <div className="habits-empty">
          <div className="empty-icon">🌱</div>
          <h3>No habits yet</h3>
          <p>Start building your atomic habits today!</p>
          <button
            type="button"
            className="btn-primary"
            onClick={() => setShowAddModal(true)}
          >
            Create Your First Habit
          </button>
        </div>
      )}

      {/* Habits Grid */}
      {habits.length > 0 && (
        <div className="habits-grid-container">
          {/* Days Header */}
          <div className="habits-grid-header">
            <div className="habit-info-header">Habit</div>
            <div className="days-header">
              {Array.from({ length: daysInMonth }, (_, i) => {
                const day = i + 1;
                const isTodayCell = isToday(year, month, day);
                return (
                  <div
                    key={day}
                    className={`day-header ${isTodayCell ? "today" : ""}`}
                  >
                    {day}
                  </div>
                );
              })}
            </div>
          </div>

          {/* Habit Rows */}
          <div className="habits-grid-body">
            {habits.map((habit) => (
              <HabitRow
                key={habit.id}
                habitId={habit.id}
                habitName={habit.name}
                habitColor={habit.color}
                year={year}
                month={month}
                daysInMonth={daysInMonth}
                completionRate={stats.completionRates.get(habit.id) || 0}
                streak={stats.streaks.get(habit.id) || 0}
                onEdit={() => setHabitToEdit(habit)}
                onDelete={() => setHabitToDelete(habit)}
              />
            ))}
          </div>
        </div>
      )}

      {/* Chart Section */}
      {habits.length > 0 && chartData.length > 0 && (
        <div className="habits-chart-section">
          <h3 className="section-title">📊 Daily Activity</h3>
          <div className="chart-container habits-chart">
            <ResponsiveContainer width="100%" height={200}>
              <AreaChart
                data={chartData}
                margin={{ top: 10, right: 10, left: -20, bottom: 0 }}
              >
                <defs>
                  <linearGradient
                    id="habitGradient"
                    x1="0"
                    y1="0"
                    x2="0"
                    y2="1"
                  >
                    <stop offset="5%" stopColor="#8b5cf6" stopOpacity={0.8} />
                    <stop offset="95%" stopColor="#8b5cf6" stopOpacity={0.1} />
                  </linearGradient>
                </defs>
                <CartesianGrid
                  strokeDasharray="3 3"
                  stroke="rgba(139, 92, 246, 0.1)"
                />
                <XAxis
                  dataKey="day"
                  stroke="#8c93a8"
                  tick={{ fill: "#8c93a8", fontSize: 11 }}
                  axisLine={{ stroke: "rgba(139, 92, 246, 0.2)" }}
                />
                <YAxis
                  stroke="#8c93a8"
                  tick={{ fill: "#8c93a8", fontSize: 11 }}
                  axisLine={{ stroke: "rgba(139, 92, 246, 0.2)" }}
                  allowDecimals={false}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#11182b",
                    border: "1px solid rgba(139, 92, 246, 0.3)",
                    borderRadius: "8px",
                    color: "#e8ecf6",
                  }}
                  formatter={(value: number) => [
                    `${value} habit${value !== 1 ? "s" : ""}`,
                    "Completed",
                  ]}
                  labelFormatter={(label) => `Day ${label}`}
                />
                <Area
                  type="monotone"
                  dataKey="completions"
                  stroke="#8b5cf6"
                  strokeWidth={2}
                  fill="url(#habitGradient)"
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>
      )}

      {/* Add/Edit Modal */}
      <AddHabitModal
        isOpen={showAddModal || habitToEdit !== null}
        onClose={handleCloseModal}
        onSubmit={handleAddSubmit}
        form={form}
        setFormField={setFormField}
        isLoading={isLoading}
        isEdit={habitToEdit !== null}
      />

      {/* Delete Confirmation Modal */}
      <DeleteConfirmModal
        isOpen={habitToDelete !== null}
        onClose={cancelDelete}
        onConfirm={confirmDelete}
        isLoading={isLoading}
        title="Delete Habit"
        message={`Are you sure you want to delete "${habitToDelete?.name}"? This will also delete all completion history for this habit.`}
      />
    </div>
  );
};

export default HabitsView;
