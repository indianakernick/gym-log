import type { ExerciseType, ExerciseTypeGroup } from '@/model/api';

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
