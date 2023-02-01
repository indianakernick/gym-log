import type { ExerciseType, ExerciseTypeGroup, MeasurementType } from '@/model/api';

export const MEASUREMENT_TYPE: { [key in MeasurementType]: string } = {
  'weight': 'Weight (kg)',
  'height': 'Height (cm)',
  'arm-right-upper': 'Right Upper Arm (cm)',
  'arm-right-lower': 'Right Lower Arm (cm)',
  'arm-left-upper': 'Left Upper Arm (cm)',
  'arm-left-lower': 'Left Lower Arm (cm)',
  'leg-right-upper': 'Right Upper Leg (cm)',
  'leg-right-lower': 'Right Lower Leg (cm)',
  'leg-left-upper': 'Left Upper Leg (cm)',
  'leg-left-lower': 'Left Lower Leg (cm)',
};

export const EXERCISE_TYPE_GROUP: { [key in ExerciseTypeGroup]: string } = {
  arms: 'Arms',
  legs: 'Legs',
  cardio: 'Cardio',
};

export const EXERCISE_TYPE: { [key in ExerciseType]: string } = {
  'biceps-curl': 'Biceps Curl',
  'chest-press': 'Chest Press',
  'dumbbell-wrist-curl': 'Dumbbell Wrist Curl',
  'elliptical-cross-trainer': 'Elliptical Cross Trainer',
  'fixed-pulldown': 'Fixed Pulldown',
  'leg-extension': 'Leg Extension',
  'pectoral-fly': 'Pectoral Fly',
  'recumbent-bike': 'Recumbent Bike',
  'seated-row': 'Seated Row',
  'shoulder-press': 'Shoulder Press',
  'treadmill': 'Treadmill',
  'triceps-extension': 'Triceps Extension',
  'upright-bike': 'Upright Bike',
};
