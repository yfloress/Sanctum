# Rewards Feature Progress

## Estado Actual: UI Completada ✅

Este documento rastrea el progreso de la implementación del sistema de recompensas en Sanctum.

---

## Resumen de la Funcionalidad

El sistema de recompensas tiene **dos tipos principales**:

### 1. Streak Rewards (Recompensas por Racha)
- Ligadas a un hábito existente
- **Dos modos:**
  - **Consecutivo**: Días seguidos (ej: 7 días sin alcohol). Si fallas, se resetea.
  - **Acumulativo**: X días de Y totales (ej: 10 de 30 días de yoga). No se resetea.
- Múltiples milestones con recompensas (ej: 7 días → café, 21 días → juego, 50 días → viaje)
- Barra de progreso visual con marcadores

### 2. Goals (Metas)
- Independientes de hábitos
- Checkpoints que se van marcando (ej: correr 5km, 10km, 21km, 42km)
- Fecha límite opcional
- Recompensa al completar todos los checkpoints

### 3. History (Historial)
- Galería de logros/trofeos desbloqueados
- Muestra fecha de cuando se logró cada achievement

---

## Archivos Creados (UI - Slint)

### Componentes (`ui/components/rewards/`)
| Archivo | Líneas | Descripción |
|---------|--------|-------------|
| `streak_reward_card.slint` | 287 | Card con barra de progreso y milestones |
| `goal_card.slint` | 279 | Card con checkpoints y progreso |
| `achievement_card.slint` | 109 | Card de trofeo para historial |
| `habits_tab.slint` | 353 | Tab de hábitos extraído |
| `rewards_tab.slint` | 112 | Tab de rewards y goals |
| `history_tab.slint` | 148 | Tab de logros desbloqueados |

### Formularios (`ui/components/forms/`)
| Archivo | Líneas | Descripción |
|---------|--------|-------------|
| `form_fields.slint` | 233 | FormField, NumberField, DateField |
| `habit_dropdown.slint` | 129 | Selector de hábitos |

### Modales (`ui/modals/`)
| Archivo | Líneas | Descripción |
|---------|--------|-------------|
| `add_reward.slint` | 553 | Modal crear streak reward |
| `add_goal.slint` | 418 | Modal crear goal con checkpoints |

### Modificados
| Archivo | Cambios |
|---------|---------|
| `ui/globals.slint` | RewardsAdapter, structs (MilestoneData, StreakRewardData, GoalData, etc.) |
| `ui/pages/habits.slint` | Tabs (Habits/Rewards/History), imports de modales |

### Iconos Restaurados
- trophy.svg, medal.svg, flame.svg, target.svg, goal.svg, award.svg, crown.svg, flag.svg

---

## Estructuras de Datos (en globals.slint)

```slint
struct MilestoneData {
    id, target-days, reward-text, unlocked, unlocked-at
}

struct StreakRewardData {
    id, habit-id, habit-name, habit-color,
    is-consecutive, target-days, target-total,
    current-progress, milestones[], 
    next-milestone-days, next-milestone-reward, progress-percent
}

struct CheckpointData {
    id, description, completed, completed-at, sort-order
}

struct GoalData {
    id, name, description, reward-text, deadline,
    checkpoints[], completed-count, total-count,
    progress-percent, is-completed, completed-at
}

struct AchievementData {
    id, title, description, icon, achieved-at, achievement-type
}

global RewardsAdapter {
    streak-rewards[], goals[], achievements[],
    active-tab, edit-reward-id, edit-goal-id,
    callbacks: create-streak-reward, add-milestone, delete-streak-reward,
               create-goal, add-checkpoint, toggle-checkpoint, delete-goal,
               fetch-rewards, fetch-goals, fetch-achievements
}
```

---

## LO QUE FALTA: Backend (Rust)

### 1. Modelos (`src/models.rs`)
Agregar structs:
```rust
pub struct StreakReward {
    pub id: String,
    pub habit_id: String,
    pub is_consecutive: bool,
    pub target_days: Option<i32>,      // para acumulativo
    pub target_total: Option<i32>,     // para acumulativo
    pub created_at: String,
}

pub struct Milestone {
    pub id: String,
    pub reward_id: String,
    pub target_days: i32,
    pub reward_text: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}

pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub reward_text: String,
    pub deadline: Option<String>,
    pub is_completed: bool,
    pub completed_at: Option<String>,
    pub created_at: String,
}

pub struct Checkpoint {
    pub id: String,
    pub goal_id: String,
    pub description: String,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub sort_order: i32,
}

pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_path: String,
    pub achievement_type: String,  // "streak" | "goal"
    pub source_id: String,         // reward_id o goal_id
    pub achieved_at: String,
}
```

### 2. Tablas SQL (`src/db/habits.rs` o nuevo archivo)
```sql
CREATE TABLE streak_rewards (
    id TEXT PRIMARY KEY,
    habit_id TEXT NOT NULL REFERENCES habits(id),
    is_consecutive INTEGER NOT NULL DEFAULT 1,
    target_days INTEGER,
    target_total INTEGER,
    created_at TEXT NOT NULL
);

CREATE TABLE milestones (
    id TEXT PRIMARY KEY,
    reward_id TEXT NOT NULL REFERENCES streak_rewards(id),
    target_days INTEGER NOT NULL,
    reward_text TEXT NOT NULL,
    unlocked INTEGER NOT NULL DEFAULT 0,
    unlocked_at TEXT
);

CREATE TABLE goals (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    reward_text TEXT NOT NULL,
    deadline TEXT,
    is_completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY,
    goal_id TEXT NOT NULL REFERENCES goals(id),
    description TEXT NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE achievements (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    icon_path TEXT NOT NULL,
    achievement_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    achieved_at TEXT NOT NULL
);
```

### 3. Repository (`src/features/habits/repository.rs`)
- CRUD para streak_rewards, milestones, goals, checkpoints, achievements
- Queries especiales:
  - `get_streak_rewards_with_progress()` - calcular progreso actual
  - `get_goals_with_checkpoints()` - cargar goals con sus checkpoints
  - `get_achievements()` - todos los logros desbloqueados

### 4. Service (`src/features/habits/service.rs`)
Lógica de negocio:
- `create_streak_reward(habit_id, is_consecutive, target_days, target_total)`
- `add_milestone(reward_id, days, reward_text)`
- `update_streak_progress(habit_id)` - llamar cuando se hace toggle de habit
- `check_and_unlock_milestones(reward_id)` - verificar si se desbloquea algo
- `create_goal(name, description, reward, deadline)`
- `add_checkpoint(goal_id, description)`
- `toggle_checkpoint(goal_id, checkpoint_id)`
- `complete_goal(goal_id)` - cuando todos los checkpoints están completos
- `create_achievement(...)` - crear trofeo cuando se desbloquea algo

### 5. Controller (`src/controller/habits.rs`)
Orquestar llamadas del service.

### 6. Callbacks UI (`src/ui/callbacks/`)
Conectar RewardsAdapter con el backend:
```rust
// En el archivo de callbacks de habits
rewards_adapter.on_create_streak_reward(...)
rewards_adapter.on_add_milestone(...)
rewards_adapter.on_fetch_rewards(...)
rewards_adapter.on_fetch_goals(...)
rewards_adapter.on_fetch_achievements(...)
// etc.
```

### 7. Migración de DB
Agregar las tablas nuevas al schema de inicialización.

---

## Orden Recomendado de Implementación

1. ✅ UI Slint (COMPLETADO)
2. ⬜ Agregar modelos en `src/models.rs`
3. ⬜ Crear tablas SQL y migración
4. ⬜ Implementar repository
5. ⬜ Implementar service con lógica de negocio
6. ⬜ Conectar callbacks UI
7. ⬜ Testing

---

## Commit Actual
```
13351a6 feat(habits): add rewards system UI with tabs, modals and components
```

## Branch
`feature/slint-migration`

---

## Notas Importantes

- **Límite de líneas**: Mantener archivos < 600 líneas
- **Patrones del proyecto**: Seguir feature-sliced (controller → service → repository → db)
- **SQLCipher**: La DB usa encriptación, no loguear datos sensibles
- **Comandos**: Usar `nix develop -c cargo ...` para build/test/run