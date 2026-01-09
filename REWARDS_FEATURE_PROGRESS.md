# Rewards Feature Progress

## Estado Actual: Backend Completado ✅

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

## ✅ Archivos Creados - UI (Slint)

### Componentes (`ui/components/rewards/`)
| Archivo | Descripción |
|---------|-------------|
| `streak_reward_card.slint` | Card con barra de progreso y milestones |
| `goal_card.slint` | Card con checkpoints y progreso |
| `achievement_card.slint` | Card de trofeo para historial |
| `habits_tab.slint` | Tab de hábitos extraído |
| `rewards_tab.slint` | Tab de rewards y goals |
| `history_tab.slint` | Tab de logros desbloqueados |

### Formularios (`ui/components/forms/`)
| Archivo | Descripción |
|---------|-------------|
| `form_fields.slint` | FormField, NumberField, DateField |
| `habit_dropdown.slint` | Selector de hábitos |

### Modales (`ui/modals/`)
| Archivo | Descripción |
|---------|-------------|
| `add_reward.slint` | Modal crear streak reward |
| `add_goal.slint` | Modal crear goal con checkpoints |

### Modificados
| Archivo | Cambios |
|---------|---------|
| `ui/globals.slint` | RewardsAdapter, structs (MilestoneData, StreakRewardData, GoalData, etc.) |
| `ui/pages/habits.slint` | Tabs (Habits/Rewards/History), imports de modales |
| `ui/app.slint` | Re-export de RewardsAdapter |

---

## ✅ Archivos Creados - Backend (Rust)

### Modelos (`src/models.rs`)
| Struct | Descripción |
|--------|-------------|
| `StreakReward` | Recompensa por racha ligada a hábito |
| `Milestone` | Hito dentro de una recompensa |
| `Goal` | Meta independiente con checkpoints |
| `Checkpoint` | Punto de control dentro de una meta |
| `Achievement` | Logro/trofeo desbloqueado |

### Base de Datos (`src/db/`)
| Archivo | Descripción |
|---------|-------------|
| `rewards.rs` | CRUD para todas las tablas de rewards |
| `mod.rs` | Tablas SQL agregadas en `create_rewards_tables()` |

**Tablas creadas:**
- `streak_rewards` - Recompensas por racha
- `milestones` - Hitos de recompensas
- `goals` - Metas independientes
- `checkpoints` - Checkpoints de metas
- `achievements` - Logros desbloqueados

### Repository (`src/features/habits/`)
| Archivo | Descripción |
|---------|-------------|
| `rewards_repository.rs` | Capa repository para rewards |

### Service (`src/features/habits/`)
| Archivo | Descripción |
|---------|-------------|
| `rewards_service.rs` | Lógica de negocio para rewards |

**Funciones principales:**
- `create_streak_reward()` - Crear recompensa por racha
- `add_milestone()` - Agregar hito a recompensa
- `check_and_unlock_milestones()` - Verificar y desbloquear hitos
- `get_streak_progress()` - Obtener progreso actual
- `create_goal()` - Crear meta
- `add_checkpoint()` - Agregar checkpoint
- `toggle_checkpoint()` - Marcar/desmarcar checkpoint
- `complete_goal()` - Completar meta y crear achievement
- `get_achievements()` - Obtener logros

### Controller (`src/controller/`)
| Archivo | Descripción |
|---------|-------------|
| `rewards.rs` | Métodos del controller para rewards |
| `mod.rs` | Agregado `RewardsService` al `AppController` |

### Callbacks UI (`src/ui/callbacks/habits/`)
| Archivo | Descripción |
|---------|-------------|
| `rewards.rs` | Callbacks para RewardsAdapter |
| `mod.rs` | Export de `setup_rewards_callbacks` |

### Main (`src/main.rs`)
- Agregada llamada a `setup_rewards_callbacks()`

---

## Estructuras de Datos

### Slint (globals.slint)
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
```

### Rust (models.rs)
```rust
pub struct StreakReward {
    pub id: String,
    pub habit_id: String,
    pub is_consecutive: bool,
    pub target_days: Option<i32>,
    pub target_total: Option<i32>,
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
    pub achievement_type: String,
    pub source_id: String,
    pub achieved_at: String,
}
```

---

## ⬜ Lo que Falta: Pruebas y Polish

### Testing
- [ ] Unit tests para `rewards_service.rs`
- [ ] Tests de integración para repository
- [ ] Tests para cálculo de streaks consecutivos/acumulativos

### UI Polish
- [ ] Verificar que los modales funcionen correctamente
- [ ] Agregar animaciones de celebración al desbloquear logros
- [ ] Verificar responsive en diferentes tamaños de pantalla

### Integración
- [ ] Llamar `update_streak_progress` cuando se hace toggle de habit
- [ ] Mostrar notificación cuando se desbloquea un milestone

---

## Build & Test Commands

```bash
nix develop -c cargo check   # Verificar compilación
nix develop -c cargo clippy  # Linter
nix develop -c cargo test    # Tests
nix develop -c cargo run     # Ejecutar
```

---

## Branch
`feature/slint-migration`

## Commits
- `13351a6` - feat(habits): add rewards system UI with tabs, modals and components
- `0eabd68` - docs: add rewards feature progress tracking document
- (pending) - feat(habits): add rewards backend with models, db, service, and callbacks