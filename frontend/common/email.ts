export const validateThuEmail = /^[a-zA-Z][a-zA-Z-]{1,}\d{2}$/;

export const emailSuffix = [
  { value: 'THU', label: '@mails.tsinghua.edu.cn', pattern: validateThuEmail },
  { value: 'PKU', label: '@pku.edu.cn', pattern: validateThuEmail },
];
