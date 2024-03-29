import type { ExerciseType, ExerciseTypeGroup, MeasurementType } from '@/model/api';

export const MEASUREMENT_TYPE: { [key in MeasurementType]: string } = {
  'weight': 'Weight',
  'height': 'Height',
  'arm-right-upper': 'Right Bicep',
  'arm-right-lower': 'Right Forearm',
  'arm-left-upper': 'Left Bicep',
  'arm-left-lower': 'Left Forearm',
  'leg-right-upper': 'Right Thigh',
  'leg-right-lower': 'Right Calf',
  'leg-left-upper': 'Left Thigh',
  'leg-left-lower': 'Left Calf',
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

export const MEASUREMENT_TYPE_ABBR: { [key in MeasurementType]: string } = {
  'weight': 'W',
  'height': 'H',
  'arm-right-upper': 'RB',
  'arm-right-lower': 'RF',
  'arm-left-upper': 'LB',
  'arm-left-lower': 'LF',
  'leg-right-upper': 'RT',
  'leg-right-lower': 'RC',
  'leg-left-upper': 'LT',
  'leg-left-lower': 'LC',
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
  'leg-curl': 'Leg Curl',
  'leg-extension': 'Leg Extension',
  'pectoral-fly': 'Pectoral Fly',
  'recumbent-bike': 'Recumbent Bike',
  'seated-leg-curl': 'Seated Leg Curl',
  'seated-row': 'Seated Row',
  'shoulder-press': 'Shoulder Press',
  'standing-calf': 'Standing Calf',
  'treadmill': 'Treadmill',
  'triceps-extension': 'Triceps Extension',
  'upright-bike': 'Upright Bike',
};
