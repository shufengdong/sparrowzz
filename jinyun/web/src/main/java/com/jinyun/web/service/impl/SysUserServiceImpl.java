package com.jinyun.web.service.impl;

import com.jinyun.web.dao.SysUserMapper;
import com.jinyun.web.service.SysUserService;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

import java.util.List;
import java.util.Map;

@Service
public class SysUserServiceImpl implements SysUserService {
    @Autowired
    SysUserMapper sysUserMapper;

    @Override
    public List<Map<String,Object>> findAll() {
        return sysUserMapper.selectAll();
    }
}
