package com.jinyun.web.controller;

import com.alibaba.fastjson.JSONObject;
import com.jinyun.web.annotation.UserLoginToken;
import com.jinyun.web.entity.User;
import com.jinyun.web.service.CapService;
import com.jinyun.web.service.TokenService;
import com.jinyun.web.service.UserService;
import io.swagger.annotations.Api;
import io.swagger.annotations.ApiImplicitParam;
import io.swagger.annotations.ApiImplicitParams;
import io.swagger.annotations.ApiOperation;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.multipart.MultipartFile;

import java.io.File;
import java.io.IOException;
import java.util.List;
import java.util.Map;

@Api(value = "desc of class", tags="DataImport",description = "数据导入")
@RestController
@RequestMapping("import")
public class DataImportController {
    @Autowired
    CapService capService;

    @ApiOperation("数据导入")
    @PostMapping("/upload")
    public String upload(@RequestBody MultipartFile file) throws IOException {
//        for(MultipartFile file:files){
//            String fileName = file.getOriginalFilename();
//            String target = "d:/";
//            File dest = new File(target+fileName);
//            try {
//                file.transferTo(dest);
//                System.out.println( "上传成功");;
//            } catch (IOException e) {
//                e.printStackTrace();
//            } catch (IllegalStateException e) {
//                e.printStackTrace();
//            }
//            System.out.println( "上传失败");;
//        }
        String fileName = file.getOriginalFilename();
        String target = "d:/";
        File dest = new File(target+fileName);
        try {
            file.transferTo(dest);
            return "上传成功";
        } catch (IOException e) {
            e.printStackTrace();
        } catch (IllegalStateException e) {
            e.printStackTrace();
        }
        return "上传成功";
    }


    @ApiOperation(value = "数据导入列表", notes = "")
    @RequestMapping(value = "/dataImportList",method = RequestMethod.GET)
    public Object transformerInfoDetail() {
        List result = capService.dataImportList();
        return result;
    }
}
