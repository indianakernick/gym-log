import type { ExerciseType, ExerciseTypeGroup, MeasurementType } from '@/model/api';

export const MEASUREMENT_TYPE: { [key in MeasurementType]: string } = {
  'weight': 'Weight',
  'height': 'Height',
  'arm-right-upper': 'Right Upper Arm',
  'arm-right-lower': 'Right Lower Arm',
  'arm-left-upper': 'Left Upper Arm',
  'arm-left-lower': 'Left Lower Arm',
  'leg-right-upper': 'Right Upper Leg',
  'leg-right-lower': 'Right Lower Leg',
  'leg-left-upper': 'Left Upper Leg',
  'leg-left-lower': 'Left Lower Leg',
};

export const MEASUREMENT_TYPE_UNIT: { [key in MeasurementType]: string } = {
  'weight': '(kg)',
  'height': '(cm)',
  'arm-right-upper': '(cm)',
  'arm-right-lower': '(cm)',
  'arm-left-upper': '(cm)',
  'arm-left-lower': '(cm)',
  'leg-right-upper': '(cm)',
  'leg-right-lower': '(cm)',
  'leg-left-upper': '(cm)',
  'leg-left-lower': '(cm)',
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
